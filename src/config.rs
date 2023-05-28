use crate::layout::widget::WidgetType;
use serde::{Deserialize, Serialize};
use std::env::{var, VarError};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use tui::style::Color;

const CONFIG_NAME: &str = "todo-tui.conf";

#[derive(Serialize, Deserialize)]
#[serde(remote = "Color")]
pub enum ColorDef {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb(u8, u8, u8),
    Indexed(u8),
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize, PartialEq, Debug))]
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
}

impl Config {
    pub fn default() -> Self {
        Self {
            init_widget: Self::default_widget_type(),
            active_color: Self::default_color(),
            window_title: Self::default_window_title(),
            todo_path: Self::default_todo_path(),
            archive_path: None,
        }
    }

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
        if let Err(_) = reader.read_to_string(&mut buf) {
            return Self::default();
        }
        toml::from_str(buf.as_str()).unwrap_or(Self::default())
    }

    fn default_todo_path() -> String {
        var("HOME").unwrap_or(String::from("~")) + "/todo.txt"
    }

    pub fn default_path() -> Result<String, VarError> {
        Ok(var("XDG_CONFIG_HOME")
            .or_else(|_| var("HOME").map(|home| format!("{}/.config/", home)))?
            + CONFIG_NAME)
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
}

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::layout::widget::WidgetType;
    use std::io::Result;
    use tui::style::Color;

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
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(c, deserialized);
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
