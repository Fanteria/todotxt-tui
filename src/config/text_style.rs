use serde::{Deserialize, Serialize};
use tui::style::{Color, Style};
use super::text_modifier::TextModifier;
use super::colors::opt_color;

#[derive(Serialize, Deserialize, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct TextStyle {
    #[serde(default, with = "opt_color")]
    bg: Option<Color>,
    #[serde(default, with = "opt_color")]
    fg: Option<Color>,
    modifier: Option<TextModifier>,
}

impl TextStyle {
    pub fn default() -> Self {
        Self {
            bg: None,
            fg: None,
            modifier: None,
        }
    }
}

impl Into<Style> for TextStyle {
    fn into(self) -> Style {
        let style = Style::default();
        if let Some(c) = self.bg {
            style.bg(c);
        }
        if let Some(c) = self.fg {
            style.fg(c);
        }
        if let Some(s) = self.modifier {
            style.add_modifier(s.into());
        }
        style
    }
}

