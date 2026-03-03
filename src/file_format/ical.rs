use crate::todo::ToDo;
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{NaiveDate, Utc};
use std::io::{Read, Write};
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

pub fn load_tasks<R: Read>(reader: R, todo: &mut ToDo) -> Result<()> {
    let mut text = String::new();
    std::io::BufReader::new(reader)
        .read_to_string(&mut text)
        .context("Failed to read iCalendar file")?;

    for raw in extract_raw_vtodos(&text) {
        let mut task = Task::default();

        if let Some(summary) = raw_property_value(&raw, "SUMMARY") {
            task.subject = summary.to_string();
        }

        if let Some(priority_str) = raw_property_value(&raw, "PRIORITY") {
            if let Ok(p) = priority_str.trim().parse::<u32>() {
                task.priority = ical_priority_to_todotxt(p).into();
            }
        }

        if let Some(status) = raw_property_value(&raw, "STATUS") {
            if status.trim() == "COMPLETED" {
                task.finished = true;
            }
        }
        if let Some(completed) = raw_property_value(&raw, "COMPLETED") {
            task.finished = true;
            task.finish_date = parse_ical_date(completed.trim());
        }

        if let Some(dtstart) = raw_property_value(&raw, "DTSTART") {
            task.create_date = parse_ical_date(dtstart.trim());
        }

        if let Some(due) = raw_property_value(&raw, "DUE") {
            task.due_date = parse_ical_date(due.trim());
        }

        if let Some(categories) = raw_property_value(&raw, "CATEGORIES") {
            let (projects, contexts, hashtags) = parse_categories(categories);
            task.projects = projects;
            task.contexts = contexts;
            task.hashtags = hashtags;
        }

        if let Some(uid) = raw_property_value(&raw, "UID") {
            task.tags
                .insert(ICAL_UID_TAG.to_string(), uid.trim().to_string());
        }

        task.tags
            .insert(ICAL_VTODO_TAG.to_string(), BASE64.encode(&raw));

        todo.add_task(task);
    }

    Ok(())
}

pub fn save_tasks<W: Write>(writer: &mut W, tasks: &[Task]) -> Result<()> {
    writeln!(writer, "BEGIN:VCALENDAR")?;
    writeln!(writer, "VERSION:2.0")?;
    writeln!(writer, "PRODID:-//todotxt-tui//EN")?;

    for task in tasks {
        if let Some(encoded) = task.tags.get(ICAL_VTODO_TAG) {
            // Round-trip: update the original VTODO with current task fields
            let raw = BASE64
                .decode(encoded)
                .context("Failed to decode stored VTODO")?;
            let raw = String::from_utf8(raw).context("Stored VTODO is not valid UTF-8")?;
            let updated = update_vtodo_properties(&raw, task);
            write!(writer, "{updated}")?;
        } else {
            // New task: build VTODO from scratch
            let vtodo = build_new_vtodo(task);
            write!(writer, "{vtodo}")?;
        }
    }

    writeln!(writer, "END:VCALENDAR")?;
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
    match task.create_date {
        Some(date) => set_or_update_property(&mut lines, "DTSTART", &format_ical_date(date)),
        None => remove_property(&mut lines, "DTSTART"),
    }

    // DUE
    match task.due_date {
        Some(date) => set_or_update_property(&mut lines, "DUE", &format_ical_date(date)),
        None => remove_property(&mut lines, "DUE"),
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

/// Build a new VTODO string for a task that doesn't have original iCal data.
fn build_new_vtodo(task: &Task) -> String {
    let uid = task
        .tags
        .get(ICAL_UID_TAG)
        .cloned()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
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
        lines.push(format!("DTSTART:{}", format_ical_date(date)));
    }

    if let Some(date) = task.due_date {
        lines.push(format!("DUE:{}", format_ical_date(date)));
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

    const SAMPLE_ICS: &str = "\
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

    #[test]
    fn test_load_tasks() -> Result<()> {
        let mut todo = ToDo::default();
        load_tasks(SAMPLE_ICS.as_bytes(), &mut todo)?;

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
        let mut task = Task::from_str("Buy milk +shopping @errands due:2023-06-30").unwrap();
        task.priority = 0.into(); // A

        let mut buf: Vec<u8> = Vec::new();
        save_tasks(&mut buf, &[task])?;
        let output = String::from_utf8(buf)?;

        assert!(output.contains("BEGIN:VCALENDAR"));
        assert!(output.contains("BEGIN:VTODO"));
        assert!(output.contains("SUMMARY:Buy milk"));
        assert!(output.contains("PRIORITY:1"));
        assert!(output.contains("STATUS:NEEDS-ACTION"));
        assert!(output.contains("DUE:20230630"));
        assert!(output.contains("CATEGORIES:+shopping,@errands"));
        assert!(output.contains("UID:"));
        assert!(output.contains("END:VTODO"));
        assert!(output.contains("END:VCALENDAR"));

        Ok(())
    }

    #[test]
    fn test_round_trip_preserves_properties() -> Result<()> {
        // Load
        let mut todo = ToDo::default();
        load_tasks(SAMPLE_ICS.as_bytes(), &mut todo)?;

        // Modify the pending task
        todo.pending[0].subject = "Buy organic groceries".to_string();

        // Save
        let mut buf: Vec<u8> = Vec::new();
        save_tasks(&mut buf, &todo.pending)?;
        let output = String::from_utf8(buf)?;

        // Check modified field
        assert!(output.contains("SUMMARY:Buy organic groceries"));
        // Check preserved field (DESCRIPTION was not mapped, should survive)
        assert!(
            output.contains("DESCRIPTION:Milk and eggs"),
            "DESCRIPTION should be preserved: {output}"
        );
        // Check UID is preserved
        assert!(output.contains("UID:test-uid-001"));

        Ok(())
    }

    #[test]
    fn test_round_trip_preserves_custom_properties() -> Result<()> {
        let mut todo = ToDo::default();
        load_tasks(SAMPLE_ICS.as_bytes(), &mut todo)?;

        // Save the done task (which has X-CUSTOM)
        let mut buf: Vec<u8> = Vec::new();
        save_tasks(&mut buf, &todo.done)?;
        let output = String::from_utf8(buf)?;

        assert!(
            output.contains("X-CUSTOM:preserved-value"),
            "X-CUSTOM should be preserved: {output}"
        );

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
