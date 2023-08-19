use super::colors::opt_color;
use super::text_modifier::TextModifier;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tui::style::{Color, Style};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct TextStyle {
    #[serde(default, with = "opt_color")]
    bg: Option<Color>,
    #[serde(default, with = "opt_color")]
    fg: Option<Color>,
    modifier: Option<TextModifier>,
}

const PRIORITIES: [&str; 27] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z", "empty",
];

fn priority_number(s: &str) -> Option<usize> {
    for (i, prio) in PRIORITIES.iter().enumerate() {
        if *prio == s {
            return Some(i);
        }
    }
    None
}

impl TextStyle {
    pub fn bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }

    pub fn fg(mut self, fg: Color) -> Self {
        self.fg = Some(fg);
        self
    }

    pub fn modifier(mut self, modifier: TextModifier) -> Self {
        self.modifier = Some(modifier);
        self
    }

    pub fn is_some(&self) -> bool {
        self.bg.is_some() || self.fg.is_some() || self.modifier.is_some()
    }

    pub fn combine(&self, additional: &Self) -> TextStyle {
        let mut new = self.clone();
        if let Some(bg) = additional.bg {
            new.bg = Some(bg);
        }
        if let Some(fg) = additional.fg {
            new.fg = Some(fg);
        }
        if let Some(modifier) = additional.modifier {
            new.modifier = Some(modifier);
        }
        new
    }

    pub fn get_style(&self) -> Style {
        let mut style = Style::default();
        if let Some(c) = self.bg {
            style = style.bg(c);
        }
        if let Some(c) = self.fg {
            style = style.fg(c);
        }
        if let Some(s) = self.modifier {
            style = style.add_modifier(s.into());
        }
        style
    }
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct TextStyleList(HashMap<String, TextStyle>);

impl TextStyleList {

    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn get_style(&self, index: usize) -> Style {
        let name = PRIORITIES[index];
        match self.0.get(name) {
            Some(item) => item.get_style(),
            None => TextStyle::default().get_style(),
        }
    }
}

impl Default for TextStyleList {
    fn default() -> Self {
        let mut items = HashMap::new();

        items.insert(
            String::from("A"),
            TextStyle::default().fg(Color::Red),
        );
        items.insert(
            String::from("B"),
            TextStyle::default().fg(Color::Yellow),
        );
        items.insert(
            String::from("C"),
            TextStyle::default().fg(Color::Blue),
        );

        Self(items)
    }
}
