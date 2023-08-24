mod colors;
mod text_modifier;
mod text_style;

pub use self::colors::OptionalColor;

use self::{colors::*, text_style::*};
use crate::layout::widget::widget_type::WidgetType;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::{
    env::{var, VarError},
    error::Error,
    fs::File,
    io::Read,
    time::Duration,
};
use tui::style::Color;

const CONFIG_NAME: &str = "todo-tui.toml";

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
    #[serde(default = "Config::default_autosave_duration")]
    pub autosave_duration: Duration,
    #[serde(default = "Config::default_log_file")]
    pub log_file: String,
    #[serde(default = "Config::default_log_format")]
    pub log_format: String,
    #[serde(default = "Config::default_log_level")]
    pub log_level: LevelFilter,
    #[serde(default = "Config::default_file_watcher")]
    pub file_watcher: bool,
    #[serde(default = "Config::default_list_refresh_rate")]
    pub list_refresh_rate: Duration,
    #[serde(default = "Config::default_list_shift")]
    pub list_shift: usize,
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
        if let Err(e) = reader.read_to_string(&mut buf) {
            log::error!("Cannot load config: {}", e);
            return Self::default();
        }
        match toml::from_str(buf.as_str()) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Cannot parse config: {}", e);
                Self::default()
            }
        }
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

    fn default_autosave_duration() -> Duration {
        Duration::from_secs(900)
    }

    fn default_log_file() -> String {
        String::from("log.log")
    }

    fn default_log_format() -> String {
        String::from("{d} [{h({l})}] {M}: {m}{n}")
    }

    fn default_log_level() -> LevelFilter {
        LevelFilter::Info
    }

    fn default_file_watcher() -> bool {
        true
    }

    fn default_list_refresh_rate() -> Duration {
        Duration::from_secs(5)
    }

    fn default_list_shift() -> usize {
        4
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
            autosave_duration: Self::default_autosave_duration(),
            log_file: Self::default_log_file(),
            log_format: Self::default_log_format(),
            log_level: Self::default_log_level(),
            file_watcher: Self::default_file_watcher(),
            list_refresh_rate: Self::default_list_refresh_rate(),
            list_shift: Self::default_list_shift(),
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
