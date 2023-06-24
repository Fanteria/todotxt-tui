use serde::{Deserialize, Serialize};
use tui::style::Modifier;

#[derive(Serialize, Deserialize, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum TextModifier {
    Bold,
    Italic,
    Underlined,
}

impl Into<Modifier> for TextModifier {
    fn into(self) -> Modifier {
        use TextModifier::*;
        match self {
            Bold => Modifier::BOLD,
            Italic => Modifier::ITALIC,
            Underlined => Modifier::UNDERLINED,
        }
    }
}
