mod colors;
mod defaults;
mod keycode;
mod styles;
mod text_modifier;
mod text_style;

pub use self::colors::OptionalColor;
use self::defaults::*;
pub use self::keycode::KeyCodeDef;
pub use self::styles::Styles;
pub use self::text_style::TextStyle;

use self::{colors::*, text_style::*};
use crate::{
    layout::widget::widget_type::WidgetType, todo::task_list::TaskSort, ui::EventHandlerUI,
};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env::{var, VarError},
    error::Error,
    fs::File,
    io::Read,
    time::Duration,
};
use tui::style::Color;

const CONFIG_NAME: &str = "todo-tui.toml";

/// Configuration struct for the ToDo TUI application.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct Config {
    #[serde(with = "ColorDef", default = "default_active_color")]
    pub active_color: Color,
    #[serde(default = "default_widget_type")]
    pub init_widget: WidgetType,
    #[serde(default = "default_window_title")]
    pub window_title: String,
    #[serde(default = "default_todo_path")]
    pub todo_path: String,
    pub archive_path: Option<String>,
    #[serde(default = "TextStyleList::default")]
    pub priority_colors: TextStyleList,
    #[serde(default = "default_category")]
    pub category_color: TextStyle,
    #[serde(default = "default_wrap_preview")]
    pub wrap_preview: bool,
    #[serde(default = "default_list_active_color")]
    pub list_active_color: TextStyle,
    #[serde(default = "TextStyle::default")]
    pub pending_active_color: TextStyle,
    #[serde(default = "TextStyle::default")]
    pub done_active_color: TextStyle,
    #[serde(default = "default_autosave_duration")]
    pub autosave_duration: Duration,
    #[serde(default = "default_log_file")]
    pub log_file: String,
    #[serde(default = "default_log_format")]
    pub log_format: String,
    #[serde(default = "default_log_level")]
    pub log_level: LevelFilter,
    #[serde(default = "default_file_watcher")]
    pub file_watcher: bool,
    #[serde(default = "default_list_refresh_rate")]
    pub list_refresh_rate: Duration,
    #[serde(default = "default_list_shift")]
    pub list_shift: usize,
    #[serde(default = "default_pending_sort")]
    pub pending_sort: TaskSort,
    #[serde(default = "default_done_sort")]
    pub done_sort: TaskSort,
    #[serde(default = "default_preview_format")]
    pub preview_format: String,
    #[serde(default = "default_layout")]
    pub layout: String,
    #[serde(default = "default_tasks_keybind")]
    pub tasks_keybind: EventHandlerUI,
    // pub preview_keybind: EventHandler,
    #[serde(default = "default_category_keybind")]
    pub category_keybind: EventHandlerUI,
    #[serde(default = "default_list_keybind")]
    pub list_keybind: EventHandlerUI,
    #[serde(default = "default_window_keybind")]
    pub window_keybind: EventHandlerUI,
    #[serde(default = "default_category_stype")]
    pub category_style: TextStyle,
    #[serde(default = "TextStyle::default")]
    pub projects_style: TextStyle,
    #[serde(default = "TextStyle::default")]
    pub contexts_style: TextStyle,
    #[serde(default = "TextStyle::default")]
    pub hashtags_style: TextStyle,
    #[serde(default = "default_custom_category_style")]
    pub custom_category_style: HashMap<String, TextStyle>,
}

impl Config {
    /// Loads the default configuration settings.
    ///
    /// This function first attempts to load the configuration file, and if it fails, it returns the default configuration.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded configuration (`Ok`) or an error (`Err`) if loading fails.
    pub fn load_default() -> Self {
        let load = || -> Result<Self, Box<dyn Error>> {
            Ok(Self::load_config(File::open(Self::default_path()?)?))
        };
        load().unwrap_or(Self::default())
    }

    /// Loads a configuration from a provided reader.
    ///
    /// # Parameters
    ///
    /// - `reader`: A reader implementing the `Read` trait.
    ///
    /// # Returns
    ///
    /// The loaded configuration.
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

    /// Returns the default configuration file path based on environment variables.
    ///
    /// The configuration file path is determined based on the XDG_CONFIG_HOME and HOME environment variables.
    ///
    /// # Returns
    ///
    /// A `Result` containing the default configuration file path (`Ok`) or an error (`Err`) if the path cannot be determined.
    pub fn default_path() -> Result<String, VarError> {
        Ok(var("XDG_CONFIG_HOME")
            .or_else(|_| var("HOME").map(|home| format!("{}/.config/", home)))?
            + CONFIG_NAME)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            init_widget: default_widget_type(),
            active_color: default_active_color(),
            window_title: default_window_title(),
            todo_path: default_todo_path(),
            archive_path: Option::default(),
            priority_colors: TextStyleList::default(),
            category_color: default_category(),
            wrap_preview: default_wrap_preview(),
            list_active_color: default_list_active_color(),
            pending_active_color: TextStyle::default(),
            done_active_color: TextStyle::default(),
            autosave_duration: default_autosave_duration(),
            log_file: default_log_file(),
            log_format: default_log_format(),
            log_level: default_log_level(),
            file_watcher: default_file_watcher(),
            list_refresh_rate: default_list_refresh_rate(),
            list_shift: default_list_shift(),
            pending_sort: default_pending_sort(),
            done_sort: default_done_sort(),
            preview_format: default_preview_format(),
            layout: default_layout(),
            tasks_keybind: default_tasks_keybind(),
            category_keybind: default_category_keybind(),
            list_keybind: default_list_keybind(),
            window_keybind: default_window_keybind(),
            category_style: default_category_stype(),
            projects_style: TextStyle::default(),
            contexts_style: TextStyle::default(),
            hashtags_style: TextStyle::default(),
            custom_category_style: default_custom_category_style(),
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
        assert_eq!(deserialized.window_title, default_window_title());
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
