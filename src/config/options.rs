use crate::ToDoError;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use tui::style::Modifier;

#[derive(Serialize, Deserialize, ValueEnum, Clone, Debug, PartialEq, Eq, Default)]
pub enum SetFinalDateType {
    Override,
    #[default]
    OnlyMissing,
    Never,
}

impl Display for SetFinalDateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&super::parsers::enum_debug_display_parser(&format!(
            "{:?}",
            self
        )))?;
        Ok(())
    }
}

/// Represents the possible sorting options for tasks.
#[derive(Clone, Copy, Serialize, Deserialize, Default, ValueEnum, Debug, PartialEq, Eq)]
pub enum TaskSort {
    #[default]
    None,
    Reverse,
    Priority,
    Alphanumeric,
    AlphanumericReverse,
}

impl Display for TaskSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&super::parsers::enum_debug_display_parser(&format!(
            "{:?}",
            self
        )))?;
        Ok(())
    }
}

/// Serialization and deserialization support for the TUI text modifier type.
///
/// This enum is used to serialize and deserialize TUI `Modifier` objects.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextModifier {
    Bold,
    Italic,
    Underlined,
}

impl FromStr for TextModifier {
    type Err = ToDoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bold" => Ok(Self::Bold),
            "italic" => Ok(Self::Italic),
            "underline" => Ok(Self::Underlined),
            _ => Err(ToDoError::ParseTextModifier(s.to_string())),
        }
    }
}

impl From<TextModifier> for Modifier {
    fn from(val: TextModifier) -> Self {
        use TextModifier::*;
        match val {
            Bold => Modifier::BOLD,
            Italic => Modifier::ITALIC,
            Underlined => Modifier::UNDERLINED,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ToDoRes;

    use super::*;

    #[test]
    fn fom_str_text_modifier() -> ToDoRes<()> {
        assert_eq!(TextModifier::from_str("bold")?, TextModifier::Bold);
        assert_eq!(TextModifier::from_str("iTALic")?, TextModifier::Italic);
        assert!(TextModifier::from_str("Some random data").is_err());

        Ok(())
    }

    #[test]
    fn to_text_modifier() {
        let bold = TextModifier::Bold;
        assert_eq!(Modifier::from(bold), Modifier::BOLD);

        let italic = TextModifier::Italic;
        assert_eq!(Modifier::from(italic), Modifier::ITALIC);

        let underline = TextModifier::Underlined;
        assert_eq!(Modifier::from(underline), Modifier::UNDERLINED);
    }
}