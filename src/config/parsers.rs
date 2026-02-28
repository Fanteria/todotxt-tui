use std::{num::ParseIntError, time::Duration};

/// Parses a string representing seconds into a [`Duration`].
pub fn parse_duration(arg: &str) -> Result<Duration, ParseIntError> {
    Ok(Duration::from_secs(arg.parse()?))
}

/// Converts a PascalCase enum variant name to a kebab-case string.
pub fn enum_debug_display_parser(s: &str) -> String {
    let mut result = String::new();
    let mut iter = s.chars();
    if let Some(c) = iter.next() {
        result.push_str(&c.to_lowercase().collect::<String>());
    }
    iter.for_each(|c| {
        if c.is_uppercase() {
            result.push('-');
            result.push_str(&c.to_lowercase().collect::<String>());
        } else {
            result.push(c);
        }
    });
    result
}
