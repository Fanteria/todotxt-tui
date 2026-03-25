use crate::{config::FileWorkerConfig, file_worker::file_format::FileFormatTrait, todo::ToDo};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{NaiveDate, Utc};
use icalendar::{
    Calendar, CalendarComponent, CalendarDateTime, Component, DatePerhapsTime, EventLike, Todo,
    TodoStatus,
};
use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
};
use todo_txt::{task::Simple as Task, Priority};

const ICAL_VTODO_TAG: &str = "_ical_vtodo";
const ICAL_UID_TAG: &str = "_ical_uid";

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Convert iCalendar priority (1-9, 0=undefined) to todo-txt priority (0=A, 1=B, ..., 25=Z).
fn ical_priority_to_todotxt(ical_priority: u32) -> Priority {
    match ical_priority {
        1..=4 => ical_priority as u8 - 1,
        _ => 26, // lowest (no priority)
    }
    .into()
}

/// Convert todo-txt priority to iCalendar priority.
fn todotxt_priority_to_ical(priority: u8) -> Option<u32> {
    match priority {
        0..=3 => Some(priority as u32 + 1),
        _ => None,
    }
}

/// Extract a NaiveDate from a DatePerhapsTime value.
fn naive_date_from_dpt(dpt: DatePerhapsTime) -> Option<NaiveDate> {
    match dpt {
        DatePerhapsTime::Date(d) => Some(d),
        DatePerhapsTime::DateTime(cdt) => match cdt {
            CalendarDateTime::Floating(ndt) => Some(ndt.date()),
            CalendarDateTime::Utc(dt) => Some(dt.date_naive()),
            CalendarDateTime::WithTimezone { date_time, .. } => Some(date_time.date()),
        },
    }
}

/// Apply task fields to a Todo component, updating all standard properties.
fn apply_task_to_todo(todo: &mut Todo, task: &Task) {
    todo.summary(&task.subject);

    match todotxt_priority_to_ical(task.priority.clone().into()) {
        Some(p) => todo.priority(p),
        None => todo.remove_priority(),
    };

    if task.finished {
        todo.status(TodoStatus::Completed);
        match task.finish_date {
            Some(date) => todo.completed(date.and_hms_opt(0, 0, 0).unwrap().and_utc()),
            None => todo.remove_property("COMPLETED"),
        };
    } else {
        todo.status(TodoStatus::NeedsAction);
        todo.remove_property("COMPLETED");
    }

    match task.create_date {
        Some(date) => todo.starts(date),
        None => todo.remove_starts(),
    };

    match task.due_date {
        Some(date) => todo.due(date),
        None => todo.remove_due(),
    };

    todo.remove_multi_property("CATEGORIES");
    for c in std::iter::empty()
        .chain(task.projects.iter().map(|p| format!("+{p}")))
        .chain(task.contexts.iter().map(|c| format!("@{c}")))
        .chain(task.hashtags.iter().map(|h| format!("#{h}")))
    {
        todo.add_multi_property("CATEGORIES", &c);
    }

    todo.last_modified(Utc::now());
}

/// Build a Task from a parsed Todo component.
fn extract_task_from_todo(todo: &Todo) -> Task {
    let mut tags = BTreeMap::from([(
        // Store the serialized VTODO block for round-trip property preservation.
        ICAL_VTODO_TAG.to_string(),
        BASE64.encode(todo.to_string().as_bytes()),
    )]);

    if let Some(uid) = todo.get_uid() {
        tags.insert(ICAL_UID_TAG.to_string(), uid.to_string());
    }

    let mut projects = Vec::new();
    let mut contexts = Vec::new();
    let mut hashtags = Vec::new();
    // Convention: +name → project, @name → context, #name → hashtag, plain → context.
    if let Some(cats) = todo.multi_properties().get("CATEGORIES") {
        for cat in cats.iter().map(|p| p.value()) {
            if let Some(name) = cat.strip_prefix('+') {
                projects.push(name.to_string());
            } else if let Some(name) = cat.strip_prefix('@') {
                contexts.push(name.to_string());
            } else if let Some(name) = cat.strip_prefix('#') {
                hashtags.push(name.to_string());
            } else {
                contexts.push(cat.to_string());
            }
        }
    }

    Task {
        subject: todo.get_summary().unwrap_or_default().into(),
        priority: todo
            .get_priority()
            .map(ical_priority_to_todotxt)
            .unwrap_or_else(todo_txt::Priority::lowest),
        create_date: todo.get_start().and_then(naive_date_from_dpt),
        finish_date: todo.get_completed().map(|d| d.date_naive()),
        finished: matches!(todo.get_status(), Some(TodoStatus::Completed)),
        threshold_date: None,
        due_date: todo.get_due().and_then(naive_date_from_dpt),
        contexts,
        projects,
        hashtags,
        tags,
    }
}

/// Parse a stored VTODO block, update it with the current task state, and return the
/// updated Todo component. The raw_vtodo string must be a bare `BEGIN:VTODO...END:VTODO` block.
fn update_todo_component(raw_vtodo: &str, task: &Task) -> Result<Todo> {
    let wrapped = format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//todotxt-tui//EN\r\n{}END:VCALENDAR\r\n",
        raw_vtodo
    );
    let mut calendar: Calendar = wrapped
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse stored VTODO: {:?}", e))?;

    let pos = calendar
        .components
        .iter()
        .position(|c| matches!(c, CalendarComponent::Todo(_)))
        .ok_or_else(|| anyhow::anyhow!("No VTODO found in stored data"))?;

    if let CalendarComponent::Todo(mut t) = calendar.components.remove(pos) {
        let seq = t.get_sequence().unwrap_or(0) + 1;
        t.sequence(seq);
        apply_task_to_todo(&mut t, task);
        Ok(t)
    } else {
        unreachable!()
    }
}

/// Build a new Todo component for a task that has no stored iCal data.
fn build_new_todo(task: &Task, uid: &str) -> Todo {
    let mut todo = Todo::new();
    todo.uid(uid).timestamp(Utc::now());
    apply_task_to_todo(&mut todo, task);
    todo.done()
}

// ---------------------------------------------------------------------------
// Directory-based format (vdirsyncer: one .ics file per task)
// ---------------------------------------------------------------------------

pub struct ICal {
    dir_path: PathBuf,
}

impl ICal {
    pub fn new(config: &FileWorkerConfig) -> Self {
        Self {
            dir_path: config.todo_path.clone(),
        }
    }

    /// Load tasks from a vdirsyncer-style directory: each .ics file contains one VTODO.
    fn load_tasks(dir: &Path, todo: &mut ToDo) -> Result<()> {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory {:?}", dir))?
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| format!("Failed to iterate directory {:?}", dir))?;
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("ics") {
                continue;
            }
            let calendar: Calendar = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {:?}", path))?
                .parse()
                .map_err(|e| anyhow::anyhow!("Failed to parse {:?}: {:?}", path, e))?;
            for todo_component in calendar.todos() {
                todo.add_task(extract_task_from_todo(todo_component));
            }
        }

        Ok(())
    }

    /// Save tasks to a vdirsyncer-style directory: one .ics file per task named <uid>.ics.
    /// Files in the directory that correspond to no task in `tasks` are deleted.
    fn save_tasks(dir: &Path, tasks: &[Task]) -> Result<()> {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory {:?}", dir))?;

        let mut written: HashSet<String> = HashSet::new();

        for task in tasks {
            let uid = task
                .tags
                .get(ICAL_UID_TAG)
                .cloned()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            let todo_component = match task.tags.get(ICAL_VTODO_TAG) {
                Some(encoded) => {
                    let raw = BASE64
                        .decode(encoded)
                        .context("Failed to decode stored VTODO")?;
                    let raw = String::from_utf8(raw).context("Stored VTODO is not valid UTF-8")?;
                    update_todo_component(&raw, task)?
                }
                None => build_new_todo(task, &uid),
            };

            let mut calendar = Calendar::new();
            calendar.push(todo_component);
            let filename = format!("{uid}.ics");
            let path = dir.join(&filename);
            std::fs::write(&path, calendar.to_string())
                .with_context(|| format!("Failed to write {:?}", path))?;
            written.insert(filename);
        }

        // Remove .ics files that are no longer represented in tasks.
        let dir_entries = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory {:?}", dir))?;
        for entry in dir_entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("ics") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !written.contains(name) {
                        std::fs::remove_file(&path).with_context(|| {
                            format!("Failed to remove orphaned file {:?}", path)
                        })?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl FileFormatTrait for ICal {
    fn load_tasks(&self, todo: &mut ToDo) -> Result<()> {
        Self::load_tasks(&self.dir_path, todo)
    }

    fn save_tasks(&self, todo: &ToDo) -> Result<()> {
        let mut all = todo.pending.clone();
        all.extend_from_slice(&todo.done);
        Self::save_tasks(&self.dir_path, &all)
    }
}

// ---------------------------------------------------------------------------
// Single-file format (one VCALENDAR file with multiple VTODOs)
// ---------------------------------------------------------------------------

pub struct ICalSingleFile {
    path: PathBuf,
}

impl ICalSingleFile {
    pub fn new(config: &FileWorkerConfig) -> Self {
        Self {
            path: config.todo_path.clone(),
        }
    }
}

impl FileFormatTrait for ICalSingleFile {
    fn load_tasks(&self, todo: &mut ToDo) -> Result<()> {
        let calendar: Calendar = std::fs::read_to_string(&self.path)
            .with_context(|| format!("Failed to read {:?}", self.path))?
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse {:?}: {:?}", self.path, e))?;
        for todo_component in calendar.todos() {
            todo.add_task(extract_task_from_todo(todo_component));
        }
        Ok(())
    }

    fn save_tasks(&self, todo: &ToDo) -> Result<()> {
        let mut all = todo.pending.clone();
        all.extend_from_slice(&todo.done);

        let mut calendar = Calendar::new();

        for task in &all {
            let uid = task
                .tags
                .get(ICAL_UID_TAG)
                .cloned()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            let todo_component = match task.tags.get(ICAL_VTODO_TAG) {
                Some(encoded) => {
                    let raw = BASE64
                        .decode(encoded)
                        .context("Failed to decode stored VTODO")?;
                    let raw = String::from_utf8(raw).context("Stored VTODO is not valid UTF-8")?;
                    update_todo_component(&raw, task)?
                }
                None => build_new_todo(task, &uid),
            };

            calendar.push(todo_component);
        }

        std::fs::write(&self.path, calendar.to_string())
            .with_context(|| format!("Failed to write {:?}", self.path))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /// A single .ics file content as vdirsyncer would store it.
    const SAMPLE_ICS_1: &str = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VTODO\r\n\
 UID:test-uid-001\r\n\
DTSTAMP:20230501T120000Z\r\n\
SUMMARY:Buy groceries\r\n\
PRIORITY:1\r\n\
STATUS:NEEDS-ACTION\r\n\
DUE:20230630\r\n\
DTSTART:20230430\r\n\
CATEGORIES:+shopping\r\n\
CATEGORIES:@errands\r\n\
CATEGORIES:#urgent\r\n\
DESCRIPTION:Milk and eggs\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

    const SAMPLE_ICS_2: &str = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VTODO\r\n\
 UID:test-uid-002\r\n\
DTSTAMP:20230501T120000Z\r\n\
SUMMARY:Clean house\r\n\
STATUS:COMPLETED\r\n\
COMPLETED:20230521T000000Z\r\n\
CATEGORIES:@home\r\n\
X-CUSTOM:preserved-value\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

    /// A single VCALENDAR file containing both VTODOs.
    const SAMPLE_ICS_MULTI: &str = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Test//Test//EN\r\n\
BEGIN:VTODO\r\n\
 UID:test-uid-001\r\n\
DTSTAMP:20230501T120000Z\r\n\
SUMMARY:Buy groceries\r\n\
PRIORITY:1\r\n\
STATUS:NEEDS-ACTION\r\n\
DUE:20230630\r\n\
DTSTART:20230430\r\n\
CATEGORIES:+shopping\r\n\
CATEGORIES:@errands\r\n\
CATEGORIES:#urgent\r\n\
DESCRIPTION:Milk and eggs\r\n\
END:VTODO\r\n\
BEGIN:VTODO\r\n\
 UID:test-uid-002\r\n\
DTSTAMP:20230501T120000Z\r\n\
SUMMARY:Clean house\r\n\
STATUS:COMPLETED\r\n\
COMPLETED:20230521T000000Z\r\n\
CATEGORIES:@home\r\n\
X-CUSTOM:preserved-value\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("ical_test_{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn write(&self, filename: &str, content: &str) {
            std::fs::write(self.0.join(filename), content).unwrap();
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    // --- ICal (directory) tests ---

    #[test]
    fn test_load_tasks() -> Result<()> {
        let dir = TempDir::new();
        dir.write("test-uid-001.ics", SAMPLE_ICS_1);
        dir.write("test-uid-002.ics", SAMPLE_ICS_2);

        let mut todo = ToDo::default();
        ICal::load_tasks(dir.path(), &mut todo)?;

        assert_eq!(todo.pending.len(), 1);
        assert_eq!(todo.done.len(), 1);

        // Pending task
        let pending = &todo.pending[0];
        assert_eq!(pending.subject, "Buy groceries");
        assert_eq!(pending.priority, 0); // A
        assert!(!pending.finished);
        assert_eq!(
            pending.due_date,
            Some(NaiveDate::from_ymd_opt(2023, 6, 30).unwrap())
        );
        assert_eq!(
            pending.create_date,
            Some(NaiveDate::from_ymd_opt(2023, 4, 30).unwrap())
        );
        assert_eq!(pending.projects, vec!["shopping"]);
        assert_eq!(pending.contexts, vec!["errands"]);
        assert_eq!(pending.hashtags, vec!["urgent"]);
        assert_eq!(
            pending.tags.get(ICAL_UID_TAG),
            Some(&"test-uid-001".to_string())
        );
        assert!(pending.tags.contains_key(ICAL_VTODO_TAG));

        // Done task
        let done = &todo.done[0];
        assert_eq!(done.subject, "Clean house");
        assert!(done.finished);
        assert_eq!(
            done.finish_date,
            Some(NaiveDate::from_ymd_opt(2023, 5, 21).unwrap())
        );
        assert_eq!(done.contexts, vec!["home"]);
        assert_eq!(
            done.tags.get(ICAL_UID_TAG),
            Some(&"test-uid-002".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_save_new_tasks() -> Result<()> {
        let dir = TempDir::new();
        let mut task = Task::from_str("Buy milk +shopping @errands due:2023-06-30").unwrap();
        task.priority = 0.into(); // A

        ICal::save_tasks(dir.path(), &[task])?;

        let files: Vec<_> = std::fs::read_dir(dir.path())?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("ics"))
            .collect();
        assert_eq!(files.len(), 1);

        let content = std::fs::read_to_string(files[0].path())?;
        assert!(content.contains("BEGIN:VCALENDAR"));
        assert!(content.contains("BEGIN:VTODO"));
        assert!(content.contains("SUMMARY:Buy milk"));
        assert!(content.contains("PRIORITY:1"));
        assert!(content.contains("STATUS:NEEDS-ACTION"));
        assert!(content.contains("DUE;VALUE=DATE:20230630"));
        assert!(content.contains("CATEGORIES:+shopping"));
        assert!(content.contains("CATEGORIES:@errands"));
        assert!(content.contains("UID:"));
        assert!(content.contains("END:VTODO"));
        assert!(content.contains("END:VCALENDAR"));

        Ok(())
    }

    #[test]
    fn test_round_trip_preserves_properties() -> Result<()> {
        let dir = TempDir::new();
        dir.write("test-uid-001.ics", SAMPLE_ICS_1);
        dir.write("test-uid-002.ics", SAMPLE_ICS_2);

        let mut todo = ToDo::default();
        ICal::load_tasks(dir.path(), &mut todo)?;

        todo.pending[0].subject = "Buy organic groceries".to_string();

        let mut all = todo.pending.clone();
        all.extend_from_slice(&todo.done);
        ICal::save_tasks(dir.path(), &all)?;

        let content = std::fs::read_to_string(dir.path().join("test-uid-001.ics"))?;
        assert!(content.contains("SUMMARY:Buy organic groceries"));
        assert!(
            content.contains("DESCRIPTION:Milk and eggs"),
            "DESCRIPTION should be preserved: {content}"
        );
        assert!(content.contains("UID:test-uid-001"));

        Ok(())
    }

    #[test]
    fn test_round_trip_preserves_custom_properties() -> Result<()> {
        let dir = TempDir::new();
        dir.write("test-uid-002.ics", SAMPLE_ICS_2);

        let mut todo = ToDo::default();
        ICal::load_tasks(dir.path(), &mut todo)?;

        ICal::save_tasks(dir.path(), &todo.done)?;

        let content = std::fs::read_to_string(dir.path().join("test-uid-002.ics"))?;
        assert!(
            content.contains("X-CUSTOM:preserved-value"),
            "X-CUSTOM should be preserved: {content}"
        );

        Ok(())
    }

    #[test]
    fn test_orphan_files_are_deleted() -> Result<()> {
        let dir = TempDir::new();
        dir.write("test-uid-001.ics", SAMPLE_ICS_1);
        dir.write("test-uid-002.ics", SAMPLE_ICS_2);

        let mut todo = ToDo::default();
        ICal::load_tasks(dir.path(), &mut todo)?;

        ICal::save_tasks(dir.path(), &todo.pending)?;

        assert!(dir.path().join("test-uid-001.ics").exists());
        assert!(
            !dir.path().join("test-uid-002.ics").exists(),
            "Orphaned file should be removed"
        );

        Ok(())
    }

    #[test]
    fn test_non_ics_files_ignored_on_load() -> Result<()> {
        let dir = TempDir::new();
        dir.write("test-uid-001.ics", SAMPLE_ICS_1);
        dir.write("metadata.json", r#"{"version": 1}"#);
        dir.write(".vdirsyncer_status", "status data");

        let mut todo = ToDo::default();
        ICal::load_tasks(dir.path(), &mut todo)?;

        assert_eq!(todo.pending.len() + todo.done.len(), 1);

        Ok(())
    }

    #[test]
    fn test_priority_mapping() {
        assert_eq!(ical_priority_to_todotxt(1), 0); // A
        assert_eq!(ical_priority_to_todotxt(2), 1); // B
        assert_eq!(ical_priority_to_todotxt(3), 2); // C
        assert_eq!(ical_priority_to_todotxt(4), 3); // D
        assert_eq!(ical_priority_to_todotxt(5), 26); // lowest
        assert_eq!(ical_priority_to_todotxt(0), 26); // undefined = lowest

        assert_eq!(todotxt_priority_to_ical(0), Some(1));
        assert_eq!(todotxt_priority_to_ical(1), Some(2));
        assert_eq!(todotxt_priority_to_ical(3), Some(4));
        assert_eq!(todotxt_priority_to_ical(26), None);
    }

    // --- ICalSingleFile tests ---

    #[test]
    fn test_single_file_load_tasks() -> Result<()> {
        let dir = TempDir::new();
        let path = dir.path().join("tasks.ics");
        std::fs::write(&path, SAMPLE_ICS_MULTI)?;

        let config = FileWorkerConfig {
            todo_path: path,
            ..Default::default()
        };
        let fmt = ICalSingleFile::new(&config);
        let mut todo = ToDo::default();
        fmt.load_tasks(&mut todo)?;

        assert_eq!(todo.pending.len(), 1);
        assert_eq!(todo.done.len(), 1);
        assert_eq!(todo.pending[0].subject, "Buy groceries");
        assert_eq!(todo.done[0].subject, "Clean house");

        Ok(())
    }

    #[test]
    fn test_single_file_save_new_task() -> Result<()> {
        let dir = TempDir::new();
        let path = dir.path().join("tasks.ics");

        let config = FileWorkerConfig {
            todo_path: path.clone(),
            ..Default::default()
        };
        let fmt = ICalSingleFile::new(&config);

        let task = Task::from_str("Buy milk +shopping @errands due:2023-06-30").unwrap();
        let mut todo = ToDo::default();
        todo.add_task(task);
        fmt.save_tasks(&todo)?;

        let content = std::fs::read_to_string(&path)?;
        assert!(content.contains("BEGIN:VCALENDAR"));
        assert!(content.contains("SUMMARY:Buy milk"));
        assert!(content.contains("CATEGORIES:+shopping"));
        assert!(content.contains("CATEGORIES:@errands"));
        assert!(content.contains("END:VCALENDAR"));

        Ok(())
    }

    #[test]
    fn test_single_file_round_trip_preserves_properties() -> Result<()> {
        let dir = TempDir::new();
        let path = dir.path().join("tasks.ics");
        std::fs::write(&path, SAMPLE_ICS_MULTI)?;

        let config = FileWorkerConfig {
            todo_path: path.clone(),
            ..Default::default()
        };
        let fmt = ICalSingleFile::new(&config);

        let mut todo = ToDo::default();
        fmt.load_tasks(&mut todo)?;

        todo.pending[0].subject = "Buy organic groceries".to_string();
        fmt.save_tasks(&todo)?;

        let content = std::fs::read_to_string(&path)?;
        assert!(content.contains("SUMMARY:Buy organic groceries"));
        assert!(
            content.contains("DESCRIPTION:Milk and eggs"),
            "DESCRIPTION should be preserved: {content}"
        );
        assert!(
            content.contains("X-CUSTOM:preserved-value"),
            "X-CUSTOM should be preserved: {content}"
        );

        Ok(())
    }
}
