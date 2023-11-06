use serde::{Deserialize, Serialize};
use tui::style::Modifier;

/// Serialization and deserialization support for the TUI text modifier type.
///
/// This enum is used to serialize and deserialize TUI `Modifier` objects.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum TextModifier {
    Bold,
    Italic,
    Underlined,
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
