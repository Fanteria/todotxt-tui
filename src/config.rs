mod text_style;
mod text_modifier;
mod colors;
pub use self::colors::OptionalColor;

use self::text_style::*;
use self::colors::*;
use crate::layout::widget::widget_type::WidgetType;
use serde::{Deserialize, Serialize};
use std::env::{var, VarError};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use tui::style::Color;

const CONFIG_NAME: &str = "todo-tui.conf";

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct Config {
    #[serde(with = "ColorDef", default = "Config::default_color")]
    pub active_color: Color,
    #[serde(default = "Config::default_widget_type")]
    pub init_widget: WidgetType,
    #[serde(default = "Config::default_window_title")]
    pub window_title: String,
    #[serde(default = "Config::default_todo_path")]
    pub todo_path: String,
    pub archive_path: Option<String>,
    #[serde(default = "TextStyleList::default")]
    pub priority_colors: TextStyleList,
    #[serde(default = "Config::default_category")]
    pub category_color: TextStyle,
    #[serde(default = "Config::default_wrap_preview")]
    pub wrap_preview: bool,
    #[serde(default = "Config::default_list_active_color")]
    pub list_active_color: TextStyle,
    #[serde(default = "TextStyle::default")]
    pub pending_active_color: TextStyle,
    #[serde(default = "TextStyle::default")]
    pub done_active_color: TextStyle,
}

impl Config {
    pub fn load_default() -> Self {
        let load = || -> Result<Self, Box<dyn Error>> {
            Ok(Self::load_config(File::open(Self::default_path()?)?))
        };
        load().unwrap_or(Self::default())
    }

    pub fn load_config<R>(mut reader: R) -> Self
    where
        R: Read,
    {
        let mut buf = String::default();
        if reader.read_to_string(&mut buf).is_err() {
            return Self::default();
        }
        // TODO config cannot be loaded log
        toml::from_str(buf.as_str()).unwrap_or(Self::default())
    }

    pub fn default_path() -> Result<String, VarError> {
        Ok(var("XDG_CONFIG_HOME")
            .or_else(|_| var("HOME").map(|home| format!("{}/.config/", home)))?
            + CONFIG_NAME)
    }

    fn default_todo_path() -> String {
        var("HOME").unwrap_or(String::from("~")) + "/todo.txt"
    }

    fn default_category() -> TextStyle {
        TextStyle::default().bg(Color::Blue)
    }

    fn default_color() -> Color {
        Color::Red
    }

    fn default_widget_type() -> WidgetType {
        WidgetType::List
    }

    fn default_window_title() -> String {
        String::from("ToDo tui")
    }

    fn default_wrap_preview() -> bool {
        true
    }
    
    fn default_list_active_color() -> TextStyle {
        TextStyle::default().bg(Color::LightRed)
    }


}

impl Default for Config {
    fn default() -> Self {
        Self {
            init_widget: Self::default_widget_type(),
            active_color: Self::default_color(),
            window_title: Self::default_window_title(),
            todo_path: Self::default_todo_path(),
            archive_path: None,
            priority_colors: TextStyleList::default(),
            category_color: Self::default_category(),
            wrap_preview: true,
            list_active_color: Self::default_list_active_color(),
            pending_active_color: TextStyle::default(),
            done_active_color: TextStyle::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Result;

    #[test]
    fn test_deserialization() {
        let deserialized: Config = toml::from_str(
            r#"
            active_color = "Green"
            init_widget = "Done"
        "#,
        )
        .unwrap();

        assert_eq!(deserialized.active_color, Color::Green);
        assert_eq!(deserialized.init_widget, WidgetType::Done);
        assert_eq!(deserialized.window_title, Config::default_window_title());
    }

    #[test]
    fn test_serialization() {
        let c = Config::default();
        let serialized = toml::to_string_pretty(&c).unwrap();
        println!("{}", serialized);
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        // assert_eq!(c, deserialized);
    }

    #[test]
    fn test_load() -> Result<()> {
        let s = r#"
        active_color = "Blue"
        window_title = "Title"
        todo_path = "path to todo file"
        "#;

        let c = Config::load_config(s.as_bytes());
        assert_eq!(c.active_color, Color::Blue);
        assert_eq!(c.init_widget, WidgetType::List);
        assert_eq!(c.window_title, "Title");
        assert_eq!(c.todo_path, "path to todo file");
        assert_eq!(c.archive_path, None);

        Ok(())
    }

    #[test]
    fn test_default() -> Result<()> {
        assert_eq!(Config::load_config("".as_bytes()), Config::default());

        Ok(())
    }
}
