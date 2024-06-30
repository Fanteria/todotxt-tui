mod colors;
mod keycode;
mod logger;
mod styles;
mod text_modifier;
mod text_style;
mod todo_config;
mod ui_config;
mod file_worker_config;
mod parsers;

pub use self::keycode::KeyCodeDef;
pub use self::logger::Logger;
pub use self::styles::Styles;
pub use self::styles::StylesValue;
pub use self::text_style::TextStyle;
pub use self::text_style::TextStyleList;
pub use self::todo_config::ToDoConfig;
pub use self::todo_config::SetFinalDateType;
pub use self::todo_config::TaskSort;
pub use self::ui_config::UiConfig;
pub use self::file_worker_config::FileWorkerConfig;

use self::colors::opt_color;
use crate::{
    layout::widget::widget_type::WidgetType,
    ui::{EventHandlerUI, UIEvent},
};
use clap::FromArgMatches;
use clap::Subcommand;
use clap::{arg, CommandFactory, Parser};

use clap_complete::{generate, shells::Bash};
use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{
    env::var,
    error::Error,
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};
use tui::style::Color;

#[derive(Default, Subcommand, Debug, PartialEq, Eq)]
pub enum Commands {
    #[default]
    Run,
    Autocomplete {
        path: PathBuf,
    },
    Config {
        path: PathBuf,
    },
    DefaultConfig {
        path: PathBuf,
    }
}

/// Configuration struct for the ToDo TUI application.
#[derive(Serialize, Deserialize, Default, Parser)]
#[command(author, version, about, long_about = None)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct Config {

    #[command(subcommand)]
    #[serde(skip)]
    pub command: Option<Commands>,

    /// Path to configuration file.
    #[serde(skip)]
    #[arg(short, long, value_name = "FILE")]
    config_path: Option<PathBuf>,

    // /// Generate autocomplete script to given file path.
    // #[serde(skip)]
    // #[arg(long, value_name = "FILE", help_heading = "export")]
    // generate_autocomplete: Option<PathBuf>,

    // /// Generate full configuration file for actual session
    // /// so present configuration file and command lines
    // /// options are taken in account.
    // #[serde(skip)]
    // #[arg(long, value_name = "FILE", help_heading = "export")]
    // export_config: Option<PathBuf>,

    // /// Generate configuration file with default values
    // /// to given file path.
    // #[serde(skip)]
    // #[arg(long, value_name = "FILE", help_heading = "export")]
    // export_default_config: Option<PathBuf>,

    #[serde(default, with = "opt_color")]
    #[arg(long, value_name = "COLOR")]
    active_color: Option<Color>,

    /// Widget that will be active after start of the application.
    #[arg(short, long, value_name = "WIDGET_TYPE")]
    init_widget: Option<WidgetType>,

    #[clap(flatten)]
    #[serde(flatten)]
    pub ui_config: UiConfig,

    #[arg(short, long, value_name = "FLAG")]
    wrap_preview: Option<bool>,

    #[arg(long, value_name = "TEXT_STYLE")]
    list_active_color: Option<TextStyle>,

    #[arg(long, value_name = "TEXT_STYLE")]
    pending_active_color: Option<TextStyle>,

    #[arg(long, value_name = "TEXT_STYLE")]
    done_active_color: Option<TextStyle>,

    #[clap(flatten)]
    #[serde(flatten)]
    pub file_worker_config: FileWorkerConfig,

    #[serde(flatten)]
    #[command(flatten)]
    pub logger: Logger,

    #[arg(short, long, value_name = "NUMBER")]
    list_shift: Option<usize>,

    #[serde(flatten)]
    #[command(flatten)]
    pub todo_config: ToDoConfig,

    #[arg(short, long, value_name = "STRING")]
    preview_format: Option<String>,

    #[arg(long, value_name = "STRING")]
    layout: Option<String>,

    #[clap(skip)]
    tasks_keybind: Option<EventHandlerUI>,

    #[clap(skip)]
    category_keybind: Option<EventHandlerUI>,

    #[clap(skip)]
    list_keybind: Option<EventHandlerUI>,

    #[clap(flatten)]
    #[serde(flatten)]
    pub styles: Styles,
}

impl Config {
    pub fn new() -> Self {
        let matches = <Config as CommandFactory>::command().get_matches();
        let mut config = Self::load_default().unwrap();
        config.update_from_arg_matches(&matches).unwrap();
        config
    }

    /// Loads the default configuration settings.
    ///
    /// This function first attempts to load the configuration file, and if it fails, it returns the default configuration.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded configuration (`Ok`) or an error (`Err`) if loading fails.
    pub fn load(path: &PathBuf) -> io::Result<Self> {
        Ok(Self::load_from_buffer(File::open(path)?))
    }

    pub fn load_config(&self) -> io::Result<Self> {
        match &self.config_path {
            Some(path) => Config::load(path),
            None => Self::load_default(),
        }
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
        Ok(Self::load_from_buffer(File::open(path)?))
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
    pub fn load_from_buffer<R>(mut reader: R) -> Self
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

    pub fn generate_autocomplete(path: &Path) -> Result<(), Box<dyn Error>> {
        generate(
            Bash,
            &mut Self::command(),
            env!("CARGO_PKG_NAME"),
            &mut File::create(path)?,
        );
        Ok(())
    }

    pub fn export_config(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let mut output = File::create(path)?;
        write!(output, "{}", toml::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn export_default_config(path: &Path) -> Result<(), Box<dyn Error>> {
        let mut output = File::create(path)?;
        write!(
            output,
            "{}",
            toml::to_string_pretty(&Config::default())?
        )?;
        Ok(())
    }

    pub fn get_active_color(&self) -> Color {
        self.active_color.unwrap_or(Color::Red)
    }

    pub fn get_init_widget(&self) -> WidgetType {
        self.init_widget.unwrap_or(WidgetType::List)
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

    pub fn get_list_shift(&self) -> usize {
        self.list_shift.unwrap_or(4)
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
        List: 20%,
        Preview: 80%,
    ],
    [ Direction: Vertical,
      Done: 60%,
      [ 
        Contexts: 10%,
        Projects: 90%,
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
            .unwrap_or(EventHandlerUI::new(&[
                (KeyCode::Enter, UIEvent::Select),
                (KeyCode::Backspace, UIEvent::Remove),
            ]))
    }

    pub fn get_list_keybind(&self) -> EventHandlerUI {
        self.list_keybind.clone().unwrap_or(EventHandlerUI::new(&[
            (KeyCode::Char('j'), UIEvent::ListDown),
            (KeyCode::Char('k'), UIEvent::ListUp),
            (KeyCode::Char('g'), UIEvent::ListFirst),
            (KeyCode::Char('G'), UIEvent::ListLast),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Result, time::Duration};
    use self::parsers::*;

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
        assert_eq!(deserialized.ui_config.window_title, UiConfig::default().window_title);
        // assert_eq!(deserialized.get_window_title(), "ToDo tui");
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

        let c = Config::load_from_buffer(s.as_bytes());
        assert_eq!(c.active_color, Some(Color::Blue));
        assert_eq!(c.init_widget, None);
        assert_eq!(c.get_init_widget(), WidgetType::List);
        assert_eq!(c.ui_config.window_title, String::from("Title"));
        assert_eq!(c.file_worker_config.todo_path, PathBuf::from("path to todo file"));
        assert_eq!(c.file_worker_config.archive_path, None);

        Ok(())
    }

    #[test]
    fn help_can_be_generated() {
        Config::parse();
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("1000"), Ok(Duration::from_secs(1000)));
        assert!(parse_duration("-1000").is_err());
    }

    // #[test]
    // fn test_merge() {
    //     let mut conf1 = Config::default();
    //     let mut conf2 = Config::default();
    //     conf1.todo_path = Some("path/to/todo/file".to_string());
    //     conf2.archive_path = Some("path/to/archive_path/file".to_string());
    //
    //     conf1.window_title = Some("Window title".to_string());
    //     conf2.window_title = Some("Different title".to_string());
    //
    //     let new_conf = conf1.merge(conf2);
    //     assert_eq!(new_conf.todo_path, Some("path/to/todo/file".to_string()));
    //     assert_eq!(
    //         new_conf.archive_path,
    //         Some("path/to/archive_path/file".to_string())
    //     );
    //     assert_eq!(new_conf.window_title, Some("Window title".to_string()));
    // }
}
