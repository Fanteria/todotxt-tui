use crate::ToDoError;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use tui::{style::Modifier, widgets::BorderType};

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

#[derive(Clone, Copy, Serialize, Deserialize, Default, ValueEnum, Debug, PartialEq, Eq)]
pub enum SavePolicy {
    ManualOnly,
    #[default]
    AutoSave,
    OnChange,
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

#[derive(Serialize, Deserialize, ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WidgetBorderType {
    Plain,
    #[default]
    Rounded,
    Double,
    Thick,
}

impl Display for WidgetBorderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&super::parsers::enum_debug_display_parser(&format!(
            "{:?}",
            self
        )))?;
        Ok(())
    }
}

impl From<WidgetBorderType> for BorderType {
    fn from(value: WidgetBorderType) -> Self {
        match value {
            WidgetBorderType::Plain => Self::Plain,
            WidgetBorderType::Rounded => Self::Rounded,
            WidgetBorderType::Double => Self::Double,
            WidgetBorderType::Thick => Self::Thick,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    #[test]
    fn set_final_date_type_display() {
        assert_eq!(format!("{}", SetFinalDateType::Override), "override");
        assert_eq!(format!("{}", SetFinalDateType::OnlyMissing), "only-missing");
        assert_eq!(format!("{}", SetFinalDateType::Never), "never");
    }

    #[test]
    fn task_sort_display() {
        assert_eq!(format!("{}", TaskSort::None), "none");
        assert_eq!(format!("{}", TaskSort::Reverse), "reverse");
        assert_eq!(format!("{}", TaskSort::Priority), "priority");
        assert_eq!(format!("{}", TaskSort::Alphanumeric), "alphanumeric");
        assert_eq!(
            format!("{}", TaskSort::AlphanumericReverse),
            "alphanumeric-reverse"
        );
    }

    #[test]
    fn fom_str_text_modifier() -> Result<()> {
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

    #[test]
    fn widget_border_type_display() {
        assert_eq!(format!("{}", WidgetBorderType::Plain), "plain");
        assert_eq!(format!("{}", WidgetBorderType::Rounded), "rounded");
        assert_eq!(format!("{}", WidgetBorderType::Double), "double");
        assert_eq!(format!("{}", WidgetBorderType::Thick), "thick");
    }

    #[test]
    fn widget_border_type_into_border_type() {
        assert_eq!(BorderType::from(WidgetBorderType::Plain), BorderType::Plain);
        assert_eq!(
            BorderType::from(WidgetBorderType::Rounded),
            BorderType::Rounded
        );
        assert_eq!(
            BorderType::from(WidgetBorderType::Double),
            BorderType::Double
        );
        assert_eq!(BorderType::from(WidgetBorderType::Thick), BorderType::Thick);
    }
}
