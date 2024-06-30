use std::str::FromStr;

use serde::{Deserialize, Serialize};
use tui::style::Modifier;

use crate::ToDoError;

/// Serialization and deserialization support for the TUI text modifier type.
///
/// This enum is used to serialize and deserialize TUI `Modifier` objects.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextModifier {
    Bold,
    Italic,
    Underlined,
}

// TODO coverage
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
    use super::*;

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
