use super::colors::opt_color;
use super::text_modifier::TextModifier;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tui::style::{Color, Style};

/// Represents the styling for text elements.
///
/// This struct defines the style for text elements, including background color, foreground color,
/// and text modifiers.
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

impl TextStyle {
    /// Set the background color for the text style.
    ///
    /// # Parameters
    ///
    /// - `bg`: The background color to set.
    ///
    /// # Returns
    ///
    /// A new `TextStyle` with the specified background color.
    pub fn bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }

    /// Set the foreground color for the text style.
    ///
    /// # Parameters
    ///
    /// - `fg`: The foreground color to set.
    ///
    /// # Returns
    ///
    /// A new `TextStyle` with the specified foreground color.
    pub fn fg(mut self, fg: Color) -> Self {
        self.fg = Some(fg);
        self
    }

    /// Set the text modifier for the text style.
    ///
    /// # Parameters
    ///
    /// - `modifier`: The text modifier to set.
    ///
    /// # Returns
    ///
    /// A new `TextStyle` with the specified text modifier.
    #[allow(dead_code)]
    pub fn modifier(mut self, modifier: TextModifier) -> Self {
        self.modifier = Some(modifier);
        self
    }

    /// Check if the text style has any styling properties.
    ///
    /// # Returns
    ///
    /// `true` if the text style has any styling properties (background color, foreground color, or modifier),
    /// `false` otherwise.
    #[allow(dead_code)]
    pub fn is_some(&self) -> bool {
        self.bg.is_some() || self.fg.is_some() || self.modifier.is_some()
    }

    /// Combine two text styles into a new text style.
    ///
    /// This method combines the properties of two text styles to create a new text style.
    ///
    /// # Parameters
    ///
    /// - `additional`: The additional text style to combine with the current text style.
    ///
    /// # Returns
    ///
    /// A new `TextStyle` with the combined properties.
    pub fn combine(&self, additional: &Self) -> TextStyle {
        let mut new = *self;
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

    /// Get the TUI `Style` corresponding to the text style.
    ///
    /// # Returns
    ///
    /// A TUI `Style` object representing the text style with its background color, foreground color, and modifier.
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

/// Represents a list of text styles for priorities.
///
/// This struct maintains a list of text styles for different priority levels.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct TextStyleList(HashMap<String, TextStyle>);

impl TextStyleList {
    /// Get the TUI `Style` for a specific priority index.
    ///
    /// # Parameters
    ///
    /// - `index`: The index representing the priority level.
    ///
    /// # Returns
    ///
    /// A TUI `Style` object representing the text style for the specified priority level.
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

        items.insert(String::from("A"), TextStyle::default().fg(Color::Red));
        items.insert(String::from("B"), TextStyle::default().fg(Color::Yellow));
        items.insert(String::from("C"), TextStyle::default().fg(Color::Blue));

        Self(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_style() {
        let style = TextStyle::default();
        assert_eq!(style.bg, None);
        assert_eq!(style.fg, None);
        assert_eq!(style.modifier, None);
        assert!(!style.is_some());
        let style = style
            .bg(Color::Red)
            .fg(Color::Green)
            .modifier(TextModifier::Bold);
        assert_eq!(style.bg, Some(Color::Red));
        assert_eq!(style.fg, Some(Color::Green));
        assert_eq!(style.modifier, Some(TextModifier::Bold));
        assert!(style.is_some());

        let _ = style.get_style();
    }

    #[test]
    fn combine_styles() {
        let style = TextStyle::default()
            .bg(Color::Red)
            .modifier(TextModifier::Bold)
            .combine(
                &TextStyle::default()
                    .fg(Color::Green)
                    .modifier(TextModifier::Italic),
            );
        assert_eq!(style.bg, Some(Color::Red));
        assert_eq!(style.fg, Some(Color::Green));
        assert_eq!(style.modifier, Some(TextModifier::Italic));

        let style = TextStyle::default()
            .bg(Color::Red)
            .combine(&TextStyle::default().bg(Color::Yellow));
        assert_eq!(style.bg, Some(Color::Yellow));
        assert_eq!(style.fg, None);
        assert_eq!(style.modifier, None);
    }

    #[test]
    fn text_style_list() {
        assert_eq!(format!("{:?}", TextStyleList::default().get_style(0)), "Style { fg: Some(Red), bg: None, underline_color: None, add_modifier: NONE, sub_modifier: NONE }");
    }
}
