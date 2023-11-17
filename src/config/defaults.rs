use super::TextStyle;
use crate::{
    layout::widget::widget_type::WidgetType,
    todo::task_list::TaskSort,
    ui::{EventHandlerUI, UIEvent},
};
use crossterm::event::KeyCode;
use log::LevelFilter;
use std::{collections::HashMap, env::var, time::Duration};
use tui::style::Color;

/// Returns the default configuration file path based on environment variables.
///
/// The configuration file path is determined based on the XDG_CONFIG_HOME and HOME environment variables.
///
/// # Returns
///
/// A `Result` containing the default configuration file path (`Ok`) or an error (`Err`) if the path cannot be determined.
pub fn default_config_path() -> String {
    const CONFIG_FOLDER: &str = "/.config/";
    const CONFIG_NAME: &str = "todo-tui.toml";
    var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME").map(|home| format!("{home}{CONFIG_FOLDER}")))
        .unwrap_or(String::from("~") + CONFIG_FOLDER)
        + CONFIG_NAME
}

pub fn default_todo_path() -> String {
    var("HOME").unwrap_or(String::from("~")) + "/todo.txt"
}

pub fn default_category() -> TextStyle {
    TextStyle::default().bg(Color::Blue)
}

pub fn default_active_color() -> Color {
    Color::Red
}

pub fn default_init_widget() -> WidgetType {
    WidgetType::List
}

pub fn default_window_title() -> String {
    String::from("ToDo tui")
}

pub fn default_wrap_preview() -> bool {
    true
}

pub fn default_list_active_color() -> TextStyle {
    TextStyle::default().bg(Color::LightRed)
}

pub fn default_autosave_duration() -> Duration {
    Duration::from_secs(900)
}

pub fn default_log_file() -> String {
    String::from("log.log")
}

pub fn default_log_format() -> String {
    String::from("{d} [{h({l})}] {M}: {m}{n}")
}

pub fn default_log_level() -> LevelFilter {
    LevelFilter::Info
}

pub fn default_file_watcher() -> bool {
    true
}

pub fn default_list_refresh_rate() -> Duration {
    Duration::from_secs(5)
}

pub fn default_list_shift() -> usize {
    4
}

pub fn default_pending_sort() -> TaskSort {
    TaskSort::None
}

pub fn default_done_sort() -> TaskSort {
    TaskSort::None
}

pub fn default_preview_format() -> String {
    String::from(
        "
Pending: $pending Done: $done
Subject: $subject
Priority: $priority
Create date: $create_date
Link: $link",
    )
}

pub fn default_layout() -> String {
    String::from(
        r#"
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
"#,
    )
}

pub fn default_tasks_keybind() -> EventHandlerUI {
    EventHandlerUI::new(&[
        (KeyCode::Char('U'), UIEvent::SwapUpItem),
        (KeyCode::Char('D'), UIEvent::SwapDownItem),
        (KeyCode::Char('x'), UIEvent::RemoveItem),
        (KeyCode::Char('d'), UIEvent::MoveItem),
        (KeyCode::Enter, UIEvent::Select),
    ])
}

pub fn default_category_keybind() -> EventHandlerUI {
    EventHandlerUI::new(&[(KeyCode::Enter, UIEvent::Select)])
}

pub fn default_list_keybind() -> EventHandlerUI {
    EventHandlerUI::new(&[
        (KeyCode::Char('j'), UIEvent::ListDown),
        (KeyCode::Char('k'), UIEvent::ListUp),
        (KeyCode::Char('g'), UIEvent::ListFirst),
        (KeyCode::Char('G'), UIEvent::ListLast),
    ])
}

pub fn default_window_keybind() -> EventHandlerUI {
    EventHandlerUI::new(&[
        (KeyCode::Char('q'), UIEvent::Quit),
        (KeyCode::Char('S'), UIEvent::Save),
        (KeyCode::Char('u'), UIEvent::Load),
        (KeyCode::Char('H'), UIEvent::MoveLeft),
        (KeyCode::Char('L'), UIEvent::MoveRight),
        (KeyCode::Char('K'), UIEvent::MoveUp),
        (KeyCode::Char('J'), UIEvent::MoveDown),
        (KeyCode::Char('I'), UIEvent::InsertMode),
        (KeyCode::Char('E'), UIEvent::EditMode),
    ])
}

pub fn default_category_stype() -> TextStyle {
    TextStyle::default().fg(Color::DarkGray)
}

pub fn default_custom_category_style() -> HashMap<String, TextStyle> {
    let mut custom_category_style = HashMap::new();
    custom_category_style.insert(
        String::from("+todo-tui"),
        TextStyle::default().fg(Color::LightBlue),
    );

    custom_category_style
}
