use crate::todo::ToDo;
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{NaiveDate, Utc};
use std::{collections::HashSet, path::Path};
use todo_txt::task::Simple as Task;

const ICAL_VTODO_TAG: &str = "_ical_vtodo";
const ICAL_UID_TAG: &str = "_ical_uid";

/// Extracts raw VTODO text blocks from the full iCalendar string.
fn extract_raw_vtodos(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut in_vtodo = false;
    let mut current = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "BEGIN:VTODO" {
            in_vtodo = true;
            current.clear();
        }
        if in_vtodo {
            current.push_str(line);
            current.push_str("\r\n");
        }
        if trimmed == "END:VTODO" {
            in_vtodo = false;
            results.push(current.clone());
        }
    }
    results
}

/// Extract a property value from raw VTODO lines.
fn raw_property_value<'a>(raw: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key}:");
    let prefix_param = format!("{key};");
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix(&prefix) {
            return Some(val);
        }
        // Handle properties with parameters like "DTSTART;VALUE=DATE:20230430"
        if trimmed.starts_with(&prefix_param) {
            if let Some(pos) = trimmed.find(':') {
                return Some(&trimmed[pos + 1..]);
            }
        }
    }
    None
}

/// Convert iCalendar priority (1-9, 0=undefined) to todo-txt priority (0=A, 1=B, ..., 25=Z).
fn ical_priority_to_todotxt(ical_priority: u32) -> u8 {
    match ical_priority {
        1 => 0,  // A
        2 => 1,  // B
        3 => 2,  // C
        4 => 3,  // D
        _ => 26, // lowest (no priority)
    }
}

/// Convert todo-txt priority to iCalendar priority.
fn todotxt_priority_to_ical(priority: u8) -> Option<u32> {
    match priority {
        0 => Some(1),
        1 => Some(2),
        2 => Some(3),
        3 => Some(4),
        _ => None, // no priority in iCal
    }
}

/// Parse a date string in YYYYMMDD or YYYYMMDDTHHMMSS(Z) format to NaiveDate.
fn parse_ical_date(s: &str) -> Option<NaiveDate> {
    // Try DATE-TIME format first (20230430T120000Z or 20230430T120000)
    let date_part = if s.len() >= 8 { &s[..8] } else { s };
    NaiveDate::parse_from_str(date_part, "%Y%m%d").ok()
}

/// Format a NaiveDate as iCalendar DATE (YYYYMMDD).
fn format_ical_date(date: NaiveDate) -> String {
    date.format("%Y%m%d").to_string()
}

/// Parse CATEGORIES value into projects, contexts, and hashtags.
/// Convention: +name → project, @name → context, #name → hashtag, plain → context.
fn parse_categories(categories: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut projects = Vec::new();
    let mut contexts = Vec::new();
    let mut hashtags = Vec::new();

    for cat in categories.split(',') {
        let cat = cat.trim();
        if cat.is_empty() {
            continue;
        }
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

    (projects, contexts, hashtags)
}

/// Build CATEGORIES string from projects, contexts, and hashtags.
fn build_categories(task: &Task) -> String {
    let mut parts: Vec<String> = Vec::new();
    for p in &task.projects {
        parts.push(format!("+{p}"));
    }
    for c in &task.contexts {
        parts.push(format!("@{c}"));
    }
    for h in &task.hashtags {
        parts.push(format!("#{h}"));
    }
    parts.join(",")
}

/// Parse a raw VTODO block into a Task.
fn parse_vtodo(raw: &str) -> Task {
    let mut task = Task::default();

    if let Some(summary) = raw_property_value(raw, "SUMMARY") {
        task.subject = summary.to_string();
    }

    if let Some(priority_str) = raw_property_value(raw, "PRIORITY") {
        if let Ok(p) = priority_str.trim().parse::<u32>() {
            task.priority = ical_priority_to_todotxt(p).into();
        }
    }

    if let Some(status) = raw_property_value(raw, "STATUS") {
        if status.trim() == "COMPLETED" {
            task.finished = true;
        }
    }
    if let Some(completed) = raw_property_value(raw, "COMPLETED") {
        task.finished = true;
        task.finish_date = parse_ical_date(completed.trim());
    }

    if let Some(dtstart) = raw_property_value(raw, "DTSTART") {
        task.create_date = parse_ical_date(dtstart.trim());
    }

    if let Some(due) = raw_property_value(raw, "DUE") {
        task.due_date = parse_ical_date(due.trim());
    }

    if let Some(categories) = raw_property_value(raw, "CATEGORIES") {
        let (projects, contexts, hashtags) = parse_categories(categories);
        task.projects = projects;
        task.contexts = contexts;
        task.hashtags = hashtags;
    }

    if let Some(uid) = raw_property_value(raw, "UID") {
        task.tags
            .insert(ICAL_UID_TAG.to_string(), uid.trim().to_string());
    }

    task.tags
        .insert(ICAL_VTODO_TAG.to_string(), BASE64.encode(raw.as_bytes()));

    task
}

/// Load tasks from a vdirsyncer-style directory: each .ics file contains one VTODO.
pub fn load_tasks(dir: &Path, todo: &mut ToDo) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory {:?}", dir))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("Failed to iterate directory {:?}", dir))?;
    // Sort for deterministic load order
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ics") {
            continue;
        }
        let content =
            std::fs::read_to_string(&path).with_context(|| format!("Failed to read {:?}", path))?;
        for raw in extract_raw_vtodos(&content) {
            todo.add_task(parse_vtodo(&raw));
        }
    }

    Ok(())
}

/// Wrap a VTODO block in a minimal VCALENDAR envelope.
fn wrap_in_vcalendar(vtodo: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//todotxt-tui//EN\r\n{vtodo}END:VCALENDAR\r\n"
    )
}

/// Save tasks to a vdirsyncer-style directory: one .ics file per task named <uid>.ics.
/// Files in the directory that correspond to no task in `tasks` are deleted.
pub fn save_tasks(dir: &Path, tasks: &[Task]) -> Result<()> {
    std::fs::create_dir_all(dir)
        .with_context(|| format!("Failed to create directory {:?}", dir))?;

    let mut written: HashSet<String> = HashSet::new();

    for task in tasks {
        let uid = task
            .tags
            .get(ICAL_UID_TAG)
            .cloned()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let vtodo = if let Some(encoded) = task.tags.get(ICAL_VTODO_TAG) {
            let raw = BASE64
                .decode(encoded)
                .context("Failed to decode stored VTODO")?;
            let raw = String::from_utf8(raw).context("Stored VTODO is not valid UTF-8")?;
            update_vtodo_properties(&raw, task)
        } else {
            build_new_vtodo(task, &uid)
        };

        let filename = format!("{uid}.ics");
        let path = dir.join(&filename);
        std::fs::write(&path, wrap_in_vcalendar(&vtodo))
            .with_context(|| format!("Failed to write {:?}", path))?;
        written.insert(filename);
    }

    // Remove .ics files that are no longer represented in tasks
    let dir_entries =
        std::fs::read_dir(dir).with_context(|| format!("Failed to read directory {:?}", dir))?;
    for entry in dir_entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("ics") {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !written.contains(name) {
                    std::fs::remove_file(&path)
                        .with_context(|| format!("Failed to remove orphaned file {:?}", path))?;
                }
            }
        }
    }

    Ok(())
}

/// Update properties in a raw VTODO string with current task field values.
fn update_vtodo_properties(raw: &str, task: &Task) -> String {
    let mut lines: Vec<String> = raw.lines().map(|l| l.to_string()).collect();

    set_or_update_property(&mut lines, "SUMMARY", &task.subject);

    // PRIORITY
    match todotxt_priority_to_ical(task.priority.clone().into()) {
        Some(p) => set_or_update_property(&mut lines, "PRIORITY", &p.to_string()),
        None => remove_property(&mut lines, "PRIORITY"),
    }

    // STATUS + COMPLETED
    if task.finished {
        set_or_update_property(&mut lines, "STATUS", "COMPLETED");
        if let Some(date) = task.finish_date {
            set_or_update_property(
                &mut lines,
                "COMPLETED",
                &format!("{}T000000Z", format_ical_date(date)),
            );
        }
    } else {
        set_or_update_property(&mut lines, "STATUS", "NEEDS-ACTION");
        remove_property(&mut lines, "COMPLETED");
    }

    // DTSTART
    remove_property(&mut lines, "DTSTART");
    if let Some(date) = task.create_date {
        if let Some(pos) = lines.iter().position(|l| l.trim() == "END:VTODO") {
            lines.insert(pos, format!("DTSTART;VALUE=DATE:{}", format_ical_date(date)));
        }
    }

    // DUE
    remove_property(&mut lines, "DUE");
    if let Some(date) = task.due_date {
        if let Some(pos) = lines.iter().position(|l| l.trim() == "END:VTODO") {
            lines.insert(pos, format!("DUE;VALUE=DATE:{}", format_ical_date(date)));
        }
    }

    // CATEGORIES
    let categories = build_categories(task);
    if categories.is_empty() {
        remove_property(&mut lines, "CATEGORIES");
    } else {
        set_or_update_property(&mut lines, "CATEGORIES", &categories);
    }

    // Increment SEQUENCE
    increment_sequence(&mut lines);

    // Update LAST-MODIFIED
    let now = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    set_or_update_property(&mut lines, "LAST-MODIFIED", &now);

    let mut result = lines.join("\r\n");
    if !result.ends_with("\r\n") {
        result.push_str("\r\n");
    }
    result
}

/// Build a new VTODO block for a task that doesn't have original iCal data.
fn build_new_vtodo(task: &Task, uid: &str) -> String {
    let now = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

    let mut lines = vec![
        "BEGIN:VTODO".to_string(),
        format!("UID:{uid}"),
        format!("DTSTAMP:{now}"),
        format!("SUMMARY:{}", task.subject),
    ];

    if let Some(p) = todotxt_priority_to_ical(task.priority.clone().into()) {
        lines.push(format!("PRIORITY:{p}"));
    }

    if task.finished {
        lines.push("STATUS:COMPLETED".to_string());
        if let Some(date) = task.finish_date {
            lines.push(format!("COMPLETED:{}T000000Z", format_ical_date(date)));
        }
    } else {
        lines.push("STATUS:NEEDS-ACTION".to_string());
    }

    if let Some(date) = task.create_date {
        lines.push(format!("DTSTART;VALUE=DATE:{}", format_ical_date(date)));
    }

    if let Some(date) = task.due_date {
        lines.push(format!("DUE;VALUE=DATE:{}", format_ical_date(date)));
    }

    let categories = build_categories(task);
    if !categories.is_empty() {
        lines.push(format!("CATEGORIES:{categories}"));
    }

    lines.push("END:VTODO".to_string());

    lines.join("\r\n") + "\r\n"
}

fn set_or_update_property(lines: &mut Vec<String>, key: &str, value: &str) {
    let prefix = format!("{key}:");
    // Also handle properties with parameters like "DTSTART;VALUE=DATE:20230430"
    let prefix_with_param = format!("{key};");
    for line in lines.iter_mut() {
        if line.starts_with(&prefix) || line.starts_with(&prefix_with_param) {
            *line = format!("{key}:{value}");
            return;
        }
    }
    // Insert before END:VTODO
    if let Some(pos) = lines.iter().position(|l| l.trim() == "END:VTODO") {
        lines.insert(pos, format!("{key}:{value}"));
    }
}

fn remove_property(lines: &mut Vec<String>, key: &str) {
    let prefix = format!("{key}:");
    let prefix_with_param = format!("{key};");
    lines.retain(|line| !line.starts_with(&prefix) && !line.starts_with(&prefix_with_param));
}

fn increment_sequence(lines: &mut Vec<String>) {
    let prefix = "SEQUENCE:";
    for line in lines.iter_mut() {
        if let Some(val_str) = line.strip_prefix(prefix) {
            if let Ok(val) = val_str.trim().parse::<u32>() {
                *line = format!("SEQUENCE:{}", val + 1);
                return;
            }
        }
    }
    // No SEQUENCE found, add with value 1
    if let Some(pos) = lines.iter().position(|l| l.trim() == "END:VTODO") {
        lines.insert(pos, "SEQUENCE:1".to_string());
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
CATEGORIES:+shopping,@errands,#urgent\r\n\
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

    #[test]
    fn test_load_tasks() -> Result<()> {
        let dir = TempDir::new();
        dir.write("test-uid-001.ics", SAMPLE_ICS_1);
        dir.write("test-uid-002.ics", SAMPLE_ICS_2);

        let mut todo = ToDo::default();
        load_tasks(dir.path(), &mut todo)?;

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

        save_tasks(dir.path(), &[task])?;

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
        assert!(content.contains("CATEGORIES:+shopping,@errands"));
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

        // Load
        let mut todo = ToDo::default();
        load_tasks(dir.path(), &mut todo)?;

        // Modify the pending task
        todo.pending[0].subject = "Buy organic groceries".to_string();

        // Save all tasks back
        let mut all = todo.pending.clone();
        all.extend_from_slice(&todo.done);
        save_tasks(dir.path(), &all)?;

        // Reload and check
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
        load_tasks(dir.path(), &mut todo)?;

        save_tasks(dir.path(), &todo.done)?;

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
        load_tasks(dir.path(), &mut todo)?;

        // Save only pending (done task's file should be deleted)
        save_tasks(dir.path(), &todo.pending)?;

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
        load_tasks(dir.path(), &mut todo)?;

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

    #[test]
    fn test_categories_parsing() {
        let (projects, contexts, hashtags) = parse_categories("+work,@office,#urgent,plain");
        assert_eq!(projects, vec!["work"]);
        assert_eq!(contexts, vec!["office", "plain"]);
        assert_eq!(hashtags, vec!["urgent"]);
    }

    #[test]
    fn test_categories_building() {
        let task = Task {
            projects: vec!["work".to_string()],
            contexts: vec!["office".to_string()],
            hashtags: vec!["urgent".to_string()],
            ..Default::default()
        };
        assert_eq!(build_categories(&task), "+work,@office,#urgent");
    }
}
