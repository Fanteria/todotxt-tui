use crate::error::ToDoError;

use super::colors::opt_color;
use super::text_modifier::TextModifier;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
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
        new.add_style(additional);
        new
    }

    // TODO doc comment
    pub fn add_style(&mut self, additional: &Self) {
        if let Some(bg) = additional.bg {
            self.bg = Some(bg);
        }
        if let Some(fg) = additional.fg {
            self.fg = Some(fg);
        }
        if let Some(modifier) = additional.modifier {
            self.modifier = Some(modifier);
        }
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

// TODO doc comment
impl FromStr for TextStyle {
    type Err = ToDoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ret = TextStyle::default();
        for word in s.split_whitespace() {
            if let Some(stripped) = word.strip_prefix('^') {
                match Color::from_str(stripped) {
                    Ok(c) => ret = ret.bg(c),
                    Err(_) => return Err(ToDoError::ParseTextStyle(word.to_string())),
                }
            } else if let Ok(color) = Color::from_str(word) {
                ret = ret.fg(color);
            } else if let Ok(modifier) = TextModifier::from_str(word) {
                ret = ret.modifier(modifier);
            } else {
                match TextStyleList::default().0.get(word) {
                    Some(style) => ret = ret.combine(style),
                    None => return Err(ToDoError::ParseTextStyle(word.to_string())),
                }
            }
        }

        Ok(ret)
    }
}

/// Represents a list of text styles for priorities.
///
/// This struct maintains a list of text styles for different priority levels.
#[derive(Serialize, Deserialize, Clone)]
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

impl FromStr for TextStyleList {
    type Err = ToDoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ret = HashMap::new();
        for s in s.split(",") {
            match s.find(":") {
                Some(index) => {
                    for priority in PRIORITIES {
                        let key = s[..index].trim();
                        if priority == key {
                            ret.insert(key.to_string(), TextStyle::from_str(s[..index].trim())?);
                            break;
                        }
                    }
                }
                None => todo!(), // error TODO
            }
        }

        Ok(TextStyleList(ret))
    }
}

#[cfg(test)]
mod tests {
    use crate::error::ToDoRes;

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
        let style = TextStyleList::default().get_style(0);
        assert_eq!(style.fg, Some(Color::Red));
        assert_eq!(style.bg, None);
        assert!(style.add_modifier.is_empty());
    }

    #[test]
    fn from_str() -> ToDoRes<()> {
        assert_eq!(
            TextStyle::from_str("red")?,
            TextStyle::default().fg(Color::Red)
        );
        assert_eq!(
            TextStyle::from_str("^red").unwrap(),
            TextStyle::default().bg(Color::Red)
        );
        assert_eq!(
            TextStyle::from_str("green ^blue").unwrap(),
            TextStyle::default().fg(Color::Green).bg(Color::Blue)
        );
        assert_eq!(
            TextStyle::from_str("bold").unwrap(),
            TextStyle::default().modifier(TextModifier::Bold)
        );
        assert_eq!(
            TextStyle::from_str("italic").unwrap(),
            TextStyle::default().modifier(TextModifier::Italic)
        );
        assert_eq!(
            TextStyle::from_str("underline").unwrap(),
            TextStyle::default().modifier(TextModifier::Underlined)
        );
        assert_eq!(
            TextStyle::from_str("red bold ^blue italic").unwrap(),
            TextStyle::default()
                .fg(Color::Red)
                .modifier(TextModifier::Bold)
                .bg(Color::Blue)
                .modifier(TextModifier::Italic)
        );
        Ok(())
    }

    #[test]
    fn from_str_err() {
        assert_eq!(
            TextStyle::from_str("invalid_color").unwrap_err(),
            ToDoError::ParseTextStyle("invalid_color".to_string())
        );
        assert_eq!(
            TextStyle::from_str("^bg_invalid_color").unwrap_err(),
            ToDoError::ParseTextStyle("^bg_invalid_color".to_string())
        );
        assert_eq!(
            TextStyle::from_str("invalid_modifier").unwrap_err(),
            ToDoError::ParseTextStyle("invalid_modifier".to_string())
        );
        assert_eq!(
            TextStyle::from_str("unknown_style").unwrap_err(),
            ToDoError::ParseTextStyle("unknown_style".to_string())
        );
    }

    #[test]
    fn text_style_list_from_str() -> ToDoRes<()> {
        let mut expected = HashMap::<String, TextStyle>::new();
        expected.insert("A".to_string(), TextStyle::default().fg(Color::Red));
        assert_eq!(TextStyleList::from_str("A:green")?, TextStyleList(expected));

        Ok(())
    }
}
