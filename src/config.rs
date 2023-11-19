mod colors;
mod keycode;
mod logger;
mod styles;
mod text_modifier;
mod text_style;

pub use self::keycode::KeyCodeDef;
pub use self::logger::Logger;
pub use self::styles::Styles;
pub use self::text_style::TextStyle;

use self::{colors::opt_color, text_style::*};
use crate::{
    layout::widget::widget_type::WidgetType,
    todo::task_list::TaskSort,
    ui::{EventHandlerUI, UIEvent},
};
use crossterm::event::KeyCode;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env::var, fs::File, io, io::Read, time::Duration};
use tui::style::Color;
use clap::Parser;

#[derive(Parser)]
pub struct Aux {
    priority_colors: Option<TextStyleList>,
    // autosave_duration: Option<Duration>,
    // tasks_keybind: Option<EventHandlerUI>,
}

/// Configuration struct for the ToDo TUI application.
#[derive(Serialize, Deserialize, Default)]
// #[command(author, version, about, long_about = None)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct Config {
    #[serde(default, with = "opt_color")]
    active_color: Option<Color>,
    init_widget: Option<WidgetType>,
    window_title: Option<String>,
    todo_path: Option<String>,
    archive_path: Option<String>,
    priority_colors: Option<TextStyleList>,
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
    /// Loads the default configuration settings.
    ///
    /// This function first attempts to load the configuration file, and if it fails, it returns the default configuration.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded configuration (`Ok`) or an error (`Err`) if loading fails.
    pub fn load(path: &str) -> io::Result<Self> {
        Ok(Self::load_config(File::open(path)?))
    }

    /// Returns the default configuration file path based on environment variables.
    ///
    /// The configuration file path is determined based on the XDG_CONFIG_HOME and HOME environment variables.
    ///
    /// # Returns
    ///
    /// A `Result` containing the default configuration file path (`Ok`) or an error (`Err`) if the path cannot be determined.
    pub fn load_default() -> io::Result<Self> {
        const CONFIG_FOLDER: &str = "/.config/";
        const CONFIG_NAME: &str = "todo-tui.toml";
        let path = var("XDG_CONFIG_HOME")
            .or_else(|_| var("HOME").map(|home| format!("{home}{CONFIG_FOLDER}")))
            .unwrap_or(String::from("~") + CONFIG_FOLDER)
            + CONFIG_NAME;
        Ok(Self::load_config(File::open(path)?))
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

    pub fn merge(self, other: Config) -> Self {
        Self {
            active_color: self.active_color.or(other.active_color),
            init_widget: self.init_widget.or(other.init_widget),
            window_title: self.window_title.or(other.window_title),
            todo_path: self.todo_path.or(other.todo_path),
            archive_path: self.archive_path.or(other.archive_path),
            priority_colors: self.priority_colors.or(other.priority_colors),
            wrap_preview: self.wrap_preview.or(other.wrap_preview),
            list_active_color: self.list_active_color.or(other.list_active_color),
            pending_active_color: self.pending_active_color.or(other.pending_active_color),
            done_active_color: self.done_active_color.or(other.done_active_color),
            autosave_duration: self.autosave_duration.or(other.autosave_duration),
            log_file: self.log_file.or(other.log_file),
            log_format: self.log_format.or(other.log_format),
            log_level: self.log_level.or(other.log_level),
            file_watcher: self.file_watcher.or(other.file_watcher),
            list_refresh_rate: self.list_refresh_rate.or(other.list_refresh_rate),
            list_shift: self.list_shift.or(other.list_shift),
            pending_sort: self.pending_sort.or(other.pending_sort),
            done_sort: self.done_sort.or(other.done_sort),
            preview_format: self.preview_format.or(other.preview_format),
            layout: self.layout.or(other.layout),
            tasks_keybind: self.tasks_keybind.or(other.tasks_keybind),
            category_keybind: self.category_keybind.or(other.category_keybind),
            list_keybind: self.list_keybind.or(other.list_keybind),
            window_keybind: self.window_keybind.or(other.window_keybind),
            category_style: self.category_style.or(other.category_style),
            projects_style: self.projects_style.or(other.projects_style),
            contexts_style: self.contexts_style.or(other.contexts_style),
            hashtags_style: self.hashtags_style.or(other.hashtags_style),
            custom_category_style: self.custom_category_style.or(other.custom_category_style),
        }
    }

    pub fn get_active_color(&self) -> Color {
        self.active_color.unwrap_or(Color::Red)
    }

    pub fn get_init_widget(&self) -> WidgetType {
        self.init_widget.unwrap_or(WidgetType::List)
    }

    pub fn get_window_title(&self) -> String {
        self.window_title
            .clone()
            .unwrap_or(String::from("ToDo tui"))
    }

    pub fn get_todo_path(&self) -> String {
        self.todo_path
            .clone()
            .unwrap_or(var("HOME").unwrap_or(String::from("~")) + "/todo.txt")
    }

    pub fn get_archive_path(&self) -> Option<String> {
        self.archive_path.clone()
    }

    fn get_priority_colors(&self) -> TextStyleList {
        self.priority_colors.clone().unwrap_or_default()
    }

    pub fn get_wrap_preview(&self) -> bool {
        self.wrap_preview.unwrap_or(true)
    }

    pub fn get_list_active_color(&self) -> TextStyle {
        self.list_active_color
            .unwrap_or(TextStyle::default().bg(Color::LightRed))
    }

    pub fn get_pending_active_color(&self) -> TextStyle {
        self.pending_active_color.unwrap_or_default()
    }

    pub fn get_done_active_color(&self) -> TextStyle {
        self.done_active_color.unwrap_or_default()
    }

    pub fn get_autosave_duration(&self) -> Duration {
        self.autosave_duration.unwrap_or(Duration::from_secs(900))
    }

    fn get_log_file(&self) -> String {
        self.log_file.clone().unwrap_or(String::from("log.log"))
    }

    fn get_log_format(&self) -> String {
        self.log_format
            .clone()
            .unwrap_or(String::from("{d} [{h({l})}] {M}: {m}{n}"))
    }

    fn get_log_level(&self) -> LevelFilter {
        self.log_level.unwrap_or(LevelFilter::Info)
    }

    pub fn get_file_watcher(&self) -> bool {
        self.file_watcher.unwrap_or(true)
    }

    pub fn get_list_refresh_rate(&self) -> Duration {
        self.list_refresh_rate.unwrap_or(Duration::from_secs(5))
    }

    pub fn get_list_shift(&self) -> usize {
        self.list_shift.unwrap_or(4)
    }

    pub fn get_pending_sort(&self) -> TaskSort {
        self.pending_sort.unwrap_or(TaskSort::None)
    }

    pub fn get_done_sort(&self) -> TaskSort {
        self.done_sort.unwrap_or(TaskSort::None)
    }

    pub fn get_preview_format(&self) -> String {
        self.preview_format.clone().unwrap_or(String::from(
            "Pending: $pending Done: $done
Subject: $subject
Priority: $priority
Create date: $create_date
Link: $link",
        ))
    }

    pub fn get_layout(&self) -> String {
        self.layout.clone().unwrap_or(String::from(
            "
[
    Direction: Horizontal,
    Size: 50%,
    [
        List: 50%,
        Preview,
    ],
    [ Direction: Vertical,
      Done,
      [ 
        Contexts,
        Projects,
      ],
    ],
]
",
        ))
    }

    pub fn get_tasks_keybind(&self) -> EventHandlerUI {
        self.tasks_keybind.clone().unwrap_or(EventHandlerUI::new(&[
            (KeyCode::Char('U'), UIEvent::SwapUpItem),
            (KeyCode::Char('D'), UIEvent::SwapDownItem),
            (KeyCode::Char('x'), UIEvent::RemoveItem),
            (KeyCode::Char('d'), UIEvent::MoveItem),
            (KeyCode::Enter, UIEvent::Select),
        ]))
    }

    pub fn get_category_keybind(&self) -> EventHandlerUI {
        self.category_keybind
            .clone()
            .unwrap_or(EventHandlerUI::new(&[(KeyCode::Enter, UIEvent::Select)]))
    }

    pub fn get_list_keybind(&self) -> EventHandlerUI {
        self.list_keybind.clone().unwrap_or(EventHandlerUI::new(&[
            (KeyCode::Char('j'), UIEvent::ListDown),
            (KeyCode::Char('k'), UIEvent::ListUp),
            (KeyCode::Char('g'), UIEvent::ListFirst),
            (KeyCode::Char('G'), UIEvent::ListLast),
        ]))
    }

    pub fn get_window_keybind(&self) -> EventHandlerUI {
        self.window_keybind.clone().unwrap_or(EventHandlerUI::new(&[
            (KeyCode::Char('q'), UIEvent::Quit),
            (KeyCode::Char('S'), UIEvent::Save),
            (KeyCode::Char('u'), UIEvent::Load),
            (KeyCode::Char('H'), UIEvent::MoveLeft),
            (KeyCode::Char('L'), UIEvent::MoveRight),
            (KeyCode::Char('K'), UIEvent::MoveUp),
            (KeyCode::Char('J'), UIEvent::MoveDown),
            (KeyCode::Char('I'), UIEvent::InsertMode),
            (KeyCode::Char('E'), UIEvent::EditMode),
        ]))
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
        let default = || {
            let mut custom_category_style = HashMap::new();
            custom_category_style.insert(
                String::from("+todo-tui"),
                TextStyle::default().fg(Color::LightBlue),
            );
            custom_category_style
        };
        self.custom_category_style.clone().unwrap_or_else(default)
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

        assert_eq!(deserialized.active_color, Some(Color::Green));
        assert_eq!(deserialized.init_widget, Some(WidgetType::Done));
        assert_eq!(deserialized.window_title, None);
        assert_eq!(deserialized.get_window_title(), "ToDo tui");
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
        assert_eq!(c.active_color, Some(Color::Blue));
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
