use crate::{
    layout::widget::widget_type::WidgetType,
    ui::{EventHandlerUI, UIEvent},
    ToDoRes,
};
use clap::{builder::styling::AnsiColor, FromArgMatches, ValueEnum};
use crossterm::event::KeyCode;
use log::LevelFilter;
use macros::{Conf, ConfMerge};
use serde::{Deserialize, Serialize};
use std::{env::var, fmt::Display, path::PathBuf, time::Duration};
use tui::style::Color as tuiColor;

use super::{conf::Conf, conf::ConfMerge, conf::ConfigDefaults, Color, TextStyle, TextStyleList};

#[derive(Conf)]
pub struct FileWorker {
    /// The path to your todo.txt file.
    pub todo_path: PathBuf,

    /// The path to your archive.txt file. If is not provided,
    /// finished files will be stored in your todo.txt.
    pub archive_path: Option<PathBuf>,

    #[arg(short = 'd')]
    /// Autosave duration (in seconds).
    pub autosave_duration: Duration,

    /// Enable file watcher for auto-reloading.
    pub file_watcher: bool,
}

impl Default for FileWorker {
    fn default() -> Self {
        Self {
            todo_path: PathBuf::from(var("HOME").unwrap_or(String::from("~")) + "/todo.txt"),
            archive_path: None,
            autosave_duration: Duration::from_secs(900),
            file_watcher: true,
        }
    }
}

#[derive(Conf)]
pub struct ActiveColor {
    /// Color for the active list item.
    pub list_active_color: TextStyle,

    /// Color for active pending task.
    pub pending_active_color: TextStyle,

    /// Color for active completed task.
    pub done_active_color: TextStyle,
}

impl Default for ActiveColor {
    fn default() -> Self {
        Self {
            list_active_color: TextStyle::default().bg(Color::lightred()),
            pending_active_color: TextStyle::default(),
            done_active_color: TextStyle::default(),
        }
    }
}

#[derive(Conf)]
pub struct List {
    /// Indentation level for lists.
    pub list_shift: usize,

    /// List keybindings.
    pub list_keybind: EventHandlerUI,
}

impl Default for List {
    fn default() -> Self {
        Self {
            list_shift: 4,
            list_keybind: EventHandlerUI::new(&[
                (KeyCode::Char('j'), UIEvent::ListDown),
                (KeyCode::Char('k'), UIEvent::ListUp),
                (KeyCode::Char('g'), UIEvent::ListFirst),
                (KeyCode::Char('G'), UIEvent::ListLast),
            ]),
        }
    }
}

#[derive(Conf)]
pub struct Preview {
    /// Preview format (uses placeholders).
    #[arg(hide_default_value = true)]
    pub preview_format: String,

    /// Wrap long lines in the preview.
    pub wrap_preview: bool,
}

impl Default for Preview {
    fn default() -> Self {
        Self {
            preview_format: String::from(
                "Pending: $pending Done: $done
Subject: $subject
Priority: $priority
Create date: $create_date
Link: $link",
            ),
            wrap_preview: true,
        }
    }
}

#[derive(Serialize, Deserialize, ValueEnum, Clone, Debug, PartialEq, Eq, Default)]
pub enum SetFinalDateType {
    Override,
    #[default]
    OnlyMissing,
    Never,
}

impl Display for SetFinalDateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&super::parsers::enum_debug_display_parser(&format!(
            "{:?}",
            self
        )))?;
        Ok(())
    }
}

/// Represents the possible sorting options for tasks.
#[derive(Clone, Copy, Serialize, Deserialize, Default, ValueEnum, Debug, PartialEq, Eq)]
pub enum TaskSort {
    #[default]
    None,
    Reverse,
    Priority,
    Alphanumeric,
    AlphanumericReverse,
}

#[derive(Conf)]
pub struct ToDo {
    /// Add projects, contexts and tags of done tasks
    /// to list of projects, contexts and tags
    pub use_done: bool,
    /// Sorting option for pending tasks.
    pub pending_sort: TaskSort,
    /// Sorting option for completed tasks.
    pub done_sort: TaskSort,

    pub delete_final_date: bool,

    pub set_final_date: SetFinalDateType,
}

impl Default for ToDo {
    fn default() -> Self {
        Self {
            use_done: false,
            pending_sort: TaskSort::default(),
            done_sort: TaskSort::default(),
            delete_final_date: true,
            set_final_date: SetFinalDateType::default(),
        }
    }
}

#[derive(Conf)]
pub struct Ui {
    /// Widget that will be active after start of the application.
    pub init_widget: WidgetType,
    /// Title of window with opened todotxt-tui
    #[arg(short = 'T')]
    pub window_title: String,
    /// Window keybindings.
    pub window_keybinds: EventHandlerUI,
    /// List refresh rate (in seconds).
    #[arg(short = 'L')]
    pub list_refresh_rate: Duration,

    pub save_state_path: Option<PathBuf>, // TODO at now unused
    /// Layout configuration.
    #[arg(hide_default_value = true)]
    pub layout: String,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            init_widget: WidgetType::List,
            window_title: String::from("ToDo tui"),
            window_keybinds: EventHandlerUI::new(&[
                (KeyCode::Char('q'), UIEvent::Quit),
                (KeyCode::Char('S'), UIEvent::Save),
                (KeyCode::Char('u'), UIEvent::Load),
                (KeyCode::Char('H'), UIEvent::MoveLeft),
                (KeyCode::Char('L'), UIEvent::MoveRight),
                (KeyCode::Char('K'), UIEvent::MoveUp),
                (KeyCode::Char('J'), UIEvent::MoveDown),
                (KeyCode::Char('I'), UIEvent::InsertMode),
                (KeyCode::Char('E'), UIEvent::EditMode),
            ]),
            list_refresh_rate: Duration::from_secs(5),
            save_state_path: None,
            layout: String::from(concat!(
                "[",
                "  Direction: Horizontal,",
                "  Size: 50%,",
                "  [",
                "    List: 80%, Preview: 20%,",
                "  ],",
                "  [",
                "    Direction: Vertical,",
                "    Done: 60%,",
                "    [",
                "      Contexts: 50%,",
                "      Projects: 50%,",
                "    ],",
                "  ],",
                "]",
            )),
        }
    }
}

#[derive(Conf)]
pub struct WidgetBase {
    /// Task keybindings.
    pub tasks_keybind: EventHandlerUI,
    /// Category keybindings.
    pub category_keybind: EventHandlerUI,
}

impl Default for WidgetBase {
    fn default() -> Self {
        Self {
            tasks_keybind: EventHandlerUI::new(&[
                (KeyCode::Char('U'), UIEvent::SwapUpItem),
                (KeyCode::Char('D'), UIEvent::SwapDownItem),
                (KeyCode::Char('x'), UIEvent::RemoveItem),
                (KeyCode::Char('d'), UIEvent::MoveItem),
                (KeyCode::Enter, UIEvent::Select),
            ]),
            category_keybind: EventHandlerUI::new(&[
                (KeyCode::Enter, UIEvent::Select),
                (KeyCode::Backspace, UIEvent::Remove),
            ]),
        }
    }
}

#[derive(Conf)]
pub struct Logger {
    /// Path to the log file.
    log_file: PathBuf,
    /// Log format (uses placeholders)
    log_format: String,
    /// Log level.
    log_level: LevelFilter,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            log_file: PathBuf::from("log.log"),
            log_format: String::from("{d} [{h({l})}] {M}: {m}{n}"),
            log_level: LevelFilter::Info,
        }
    }
}

#[derive(Conf)]
pub struct Styles {
    /// Color of active window.
    pub active_color: Color,
    /// Priority-specific colors.
    pub priority_style: TextStyleList, // TODO incompatible option config
    /// Style for projects in lists.
    pub projects_style: TextStyle,
    /// Style for contexts in lists.
    pub contexts_style: TextStyle,
    /// Style for hashtags in lists.
    pub hashtags_style: TextStyle,
    /// Style for categories in lists.
    pub category_style: TextStyle,
    /// Style for categories to filter.
    pub category_select_style: TextStyle,
    /// Style for categories filtered out.
    pub category_remove_style: TextStyle,
    // /// Custom style by name for categories.
    // TODO
    // #[clap(skip)]
    // #[serde(default = "default_custom_category_style")]
    // pub custom_category_style: HashMap<String, TextStyle>,
}

impl Default for Styles {
    fn default() -> Self {
        // let mut custom_category_style = HashMap::new();
        // custom_category_style.insert(
        //     String::from("+todo-tui"),
        //     TextStyle::default().fg(Color::lightblue()),
        // );
        Self {
            active_color: Color(tuiColor::Red),
            priority_style: TextStyleList::default(),
            projects_style: TextStyle::default(),
            contexts_style: TextStyle::default(),
            hashtags_style: TextStyle::default(),
            category_style: TextStyle::default(),
            category_select_style: TextStyle::default().fg(Color::green()),
            category_remove_style: TextStyle::default().fg(Color::red()),
            // custom_category_style: default_custom_category_style(),
        }
    }
}

#[derive(ConfMerge, Default)]
#[command(author, version, about, long_about = None)]
pub struct Configuration {
    pub ui_config: Ui,
    pub active_color_config: ActiveColor,
    pub file_worker_config: FileWorker,
    pub logger: Logger,
    pub list_config: List,
    pub todo_config: ToDo,
    pub preview_config: Preview,
    pub widget_base_config: WidgetBase,
    pub styles: Styles,
}

impl Configuration {
}

impl ConfigDefaults for Configuration {
    fn config_path() -> PathBuf {
        const CONFIG_FOLDER: &str = "/.config/";
        const CONFIG_NAME: &str = "todotxt-tui.toml";
        match var("XDG_CONFIG_HOME") {
            Ok(config_path) => PathBuf::from(config_path),
            Err(_) => PathBuf::from(var("HOME").unwrap_or(String::from("~"))).join(CONFIG_FOLDER),
        }
        .join(CONFIG_NAME)
    }

    fn help_colors() -> clap::builder::Styles {
        clap::builder::Styles::styled()
            .usage(AnsiColor::Green.on_default().bold())
            .literal(AnsiColor::Cyan.on_default().bold())
            .header(AnsiColor::Green.on_default().bold())
            .invalid(AnsiColor::Yellow.on_default())
            .error(AnsiColor::Red.on_default().bold())
            .valid(AnsiColor::Green.on_default())
            .placeholder(AnsiColor::Cyan.on_default())
    }
}
