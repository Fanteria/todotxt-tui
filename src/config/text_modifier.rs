use serde::{Deserialize, Serialize};
use tui::style::Modifier;

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
