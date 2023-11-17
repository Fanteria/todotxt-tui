mod colors;
mod defaults;
mod keycode;
mod logger;
mod styles;
mod text_modifier;
mod text_style;

pub use self::colors::OptionalColor;
use self::defaults::*;
pub use self::keycode::KeyCodeDef;
pub use self::logger::Logger;
pub use self::styles::Styles;
pub use self::text_style::TextStyle;

use self::{colors::*, text_style::*};
use crate::{
    layout::widget::widget_type::WidgetType, todo::task_list::TaskSort, ui::EventHandlerUI,
};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fs::File, io::Read, time::Duration};
use tui::style::Color;

/// Configuration struct for the ToDo TUI application.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct Config {
    #[serde(skip, default = "default_config_path")]
    config_path: String,
    #[serde(with = "ColorDef", default = "default_active_color")]
    active_color: Color,
    init_widget: Option<WidgetType>,
    window_title: Option<String>,
    todo_path: Option<String>,
    archive_path: Option<String>,
    priority_colors: Option<TextStyleList>,
    category_color: Option<TextStyle>,
    wrap_preview: Option<bool>,
    list_active_color: Option<TextStyle>,
    pending_active_color: Option<TextStyle>,
    done_active_color: Option<TextStyle>,
    autosave_duration: Option<Duration>,
    log_file: Option<String>,
    log_format: Option<String>,
    log_level: Option<LevelFilter>,
    file_watcher: Option<bool>,
    list_refresh_rate: Option<Duration>,
    list_shift: Option<usize>,
    pending_sort: Option<TaskSort>,
    done_sort: Option<TaskSort>,
    preview_format: Option<String>,
    layout: Option<String>,
    tasks_keybind: Option<EventHandlerUI>,
    // pub preview_keybind: Option<EventHandler>,
    category_keybind: Option<EventHandlerUI>,
    list_keybind: Option<EventHandlerUI>,
    window_keybind: Option<EventHandlerUI>,
    category_style: Option<TextStyle>,
    projects_style: Option<TextStyle>,
    contexts_style: Option<TextStyle>,
    hashtags_style: Option<TextStyle>,
    custom_category_style: Option<HashMap<String, TextStyle>>,
}

impl Config {
    /// Creates empty config
    // pub fn empty() -> Self {
    //     Config {
    //
    //     }
    // }

    /// Loads the default configuration settings.
    ///
    /// This function first attempts to load the configuration file, and if it fails, it returns the default configuration.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded configuration (`Ok`) or an error (`Err`) if loading fails.
    pub fn load(self) -> Self {
        let try_load = || -> Result<Self, Box<dyn Error>> {
            Ok(Self::load_config(File::open(&self.config_path)?))
        };
        try_load().unwrap_or(Self::default())
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
    fn load_config<R>(mut reader: R) -> Self
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

    fn get_config_path(&self) {}
    pub fn get_active_color(&self) -> Color {
        self.active_color
    }

    pub fn get_init_widget(&self) -> WidgetType {
        self.init_widget.unwrap_or_else(default_init_widget)
    }

    pub fn get_window_title(&self) -> String {
        self.window_title.clone().unwrap_or_else(default_window_title)
    }

    pub fn get_todo_path(&self) -> String {
        self.todo_path.clone().unwrap_or_else(default_todo_path)
    }

    pub fn get_archive_path(&self) -> Option<String> {
        self.archive_path.clone()
    }

    fn get_priority_colors(&self) -> TextStyleList {
        self.priority_colors.clone().unwrap_or_default()
    }

    fn get_category_color(&self) -> TextStyle {
        self.category_color.unwrap_or_else(default_category)
    }

    pub fn get_wrap_preview(&self) -> bool {
        self.wrap_preview.unwrap_or_else(default_wrap_preview)
    }

    pub fn get_list_active_color(&self) -> TextStyle {
        self.list_active_color
            .unwrap_or_else(default_list_active_color)
    }

    pub fn get_pending_active_color(&self) -> TextStyle {
        self.pending_active_color.unwrap_or_default()
    }

    pub fn get_done_active_color(&self) -> TextStyle {
        self.done_active_color.unwrap_or_default()
    }

    pub fn get_autosave_duration(&self) -> Duration {
        self.autosave_duration
            .unwrap_or_else(default_autosave_duration)
    }

    fn get_log_file(&self) -> String {
        self.log_file.clone().unwrap_or_else(default_log_file)
    }

    fn get_log_format(&self) -> String {
        self.log_format.clone().unwrap_or_else(default_log_format)
    }

    fn get_log_level(&self) -> LevelFilter {
        self.log_level.unwrap_or_else(default_log_level)
    }

    pub fn get_file_watcher(&self) -> bool {
        self.file_watcher.unwrap_or_else(default_file_watcher)
    }

    pub fn get_list_refresh_rate(&self) -> Duration {
        self.list_refresh_rate
            .unwrap_or_else(default_list_refresh_rate)
    }

    pub fn get_list_shift(&self) -> usize {
        self.list_shift.unwrap_or_else(default_list_shift)
    }

    pub fn get_pending_sort(&self) -> TaskSort {
        self.pending_sort.unwrap_or_else(default_pending_sort)
    }

    pub fn get_done_sort(&self) -> TaskSort {
        self.done_sort.unwrap_or_else(default_done_sort)
    }

    pub fn get_preview_format(&self) -> String {
        self.preview_format
            .clone()
            .unwrap_or_else(default_preview_format)
    }

    pub fn get_layout(&self) -> String {
        self.layout.clone().unwrap_or_else(default_layout)
    }

    pub fn get_tasks_keybind(&self) -> EventHandlerUI {
        self.tasks_keybind
            .clone()
            .unwrap_or_else(default_tasks_keybind)
    }

    pub fn get_category_keybind(&self) -> EventHandlerUI {
        self.category_keybind
            .clone()
            .unwrap_or_else(default_category_keybind)
    }

    pub fn get_list_keybind(&self) -> EventHandlerUI {
        self.list_keybind
            .clone()
            .unwrap_or_else(default_list_keybind)
    }

    pub fn get_window_keybind(&self) -> EventHandlerUI {
        self.window_keybind
            .clone()
            .unwrap_or_else(default_window_keybind)
    }

    fn get_category_style(&self) -> TextStyle {
        self.category_style.unwrap_or_default()
    }

    fn get_projects_style(&self) -> TextStyle {
        self.projects_style.unwrap_or_default()
    }

    fn get_contexts_style(&self) -> TextStyle {
        self.contexts_style.unwrap_or_default()
    }

    fn get_hashtags_style(&self) -> TextStyle {
        self.hashtags_style.unwrap_or_default()
    }

    fn get_custom_category_style(&self) -> HashMap<String, TextStyle> {
        self.custom_category_style
            .clone()
            .unwrap_or_else(default_custom_category_style)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_path: default_config_path(),
            active_color: default_active_color(),
            init_widget: Some(default_init_widget()),
            window_title: Some(default_window_title()),
            todo_path: Some(default_todo_path()),
            archive_path: Option::default(),
            priority_colors: Some(TextStyleList::default()),
            category_color: Some(default_category()),
            wrap_preview: Some(default_wrap_preview()),
            list_active_color: Some(default_list_active_color()),
            pending_active_color: Some(TextStyle::default()),
            done_active_color: Some(TextStyle::default()),
            autosave_duration: Some(default_autosave_duration()),
            log_file: Some(default_log_file()),
            log_format: Some(default_log_format()),
            log_level: Some(default_log_level()),
            file_watcher: Some(default_file_watcher()),
            list_refresh_rate: Some(default_list_refresh_rate()),
            list_shift: Some(default_list_shift()),
            pending_sort: Some(default_pending_sort()),
            done_sort: Some(default_done_sort()),
            preview_format: Some(default_preview_format()),
            layout: Some(default_layout()),
            tasks_keybind: Some(default_tasks_keybind()),
            category_keybind: Some(default_category_keybind()),
            list_keybind: Some(default_list_keybind()),
            window_keybind: Some(default_window_keybind()),
            category_style: Some(default_category_stype()),
            projects_style: Some(TextStyle::default()),
            contexts_style: Some(TextStyle::default()),
            hashtags_style: Some(TextStyle::default()),
            custom_category_style: Some(default_custom_category_style()),
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
        assert_eq!(deserialized.init_widget, Some(WidgetType::Done));
        assert_eq!(deserialized.window_title, None);
        assert_eq!(deserialized.get_window_title(), default_window_title());
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
        assert_eq!(c.init_widget, None);
        assert_eq!(c.get_init_widget(), WidgetType::List);
        assert_eq!(c.window_title, Some(String::from("Title")));
        assert_eq!(c.todo_path, Some(String::from("path to todo file")));
        assert_eq!(c.archive_path, None);

        Ok(())
    }

    // #[test]
    // fn test_default() -> Result<()> {
    //     assert_eq!(Config::load_config("".as_bytes()), Config::default());
    //
    //     Ok(())
    // }
}
