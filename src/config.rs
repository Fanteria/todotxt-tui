mod active_color_config;
mod colors;
mod file_worker_config;
mod keycode;
mod list_config;
mod logger;
mod parsers;
mod preview_config;
mod styles;
mod text_modifier;
mod text_style;
mod todo_config;
mod ui_config;
mod widget_base_config;

pub use self::active_color_config::ActiveColorConfig;
pub use self::colors::Color;
pub use self::file_worker_config::FileWorkerConfig;
pub use self::keycode::KeyCodeDef;
pub use self::list_config::ListConfig;
pub use self::logger::Logger;
pub use self::styles::Styles;
pub use self::styles::StylesValue;
pub use self::text_style::TextStyle;
pub use self::text_style::TextStyleList;
pub use self::todo_config::SetFinalDateType;
pub use self::todo_config::TaskSort;
pub use self::todo_config::ToDoConfig;
pub use self::ui_config::UiConfig;

use crate::IOError;
use crate::ToDoIoError;
use crate::ToDoRes;
use clap::builder::styling::AnsiColor;
use clap::{CommandFactory, Parser};

use clap_complete::{generate, shells::Bash};
use preview_config::PreviewConfig;
use serde::Serialize;
use twelf::config;
use twelf::Layer;
// use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::{
    env::var,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};
use widget_base_config::WidgetBaseConfig;

pub struct Cli {}

/// Configuration struct for the ToDo TUI application.
#[config]
#[derive(Serialize, Default, Parser, PartialEq, Eq, Debug)]
#[command(author, version, about, long_about = None, styles = cli_help_style())]
pub struct Config {
    /// Generate autocomplete script to given file path.
    #[clap(long, group = "export", help_heading = "export")]
    #[serde(skip)]
    pub export_autocomplete: Option<PathBuf>,

    /// Generate full configuration file for actual session
    /// so present configuration file and command lines
    /// options are taken in account.
    #[clap(long, group = "export", help_heading = "export")]
    #[serde(skip)]
    pub export_config: Option<PathBuf>,

    /// Generate configuration file with default values
    /// to given file path.
    #[clap(long, group = "export", help_heading = "export")]
    #[serde(skip)]
    pub export_default_config: Option<PathBuf>,

    /// Path to configuration file.
    #[serde(skip)]
    #[clap(short, long)]
    config_path: Option<PathBuf>,

    #[clap(flatten)]
    #[serde(flatten)]
    pub ui_config: UiConfig,

    #[clap(flatten)]
    #[serde(flatten)]
    pub active_color_config: ActiveColorConfig,

    #[clap(flatten)]
    #[serde(flatten)]
    pub file_worker_config: FileWorkerConfig,

    #[serde(flatten)]
    #[clap(flatten)]
    pub logger: Logger,

    #[serde(flatten)]
    #[clap(flatten)]
    pub list_config: ListConfig,

    #[serde(flatten)]
    #[clap(flatten)]
    pub todo_config: ToDoConfig,

    #[serde(flatten)]
    #[clap(flatten)]
    pub preview_config: PreviewConfig,

    #[clap(flatten)]
    #[serde(flatten)]
    pub widget_base_config: WidgetBaseConfig,

    #[clap(flatten)]
    #[serde(flatten)]
    pub styles: Styles,
}

impl Config {
    pub fn new() -> ToDoRes<Self> {
        Self::from_args(env::args())
    }

    pub fn from_args<I, T>(itr: I) -> ToDoRes<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = Self::command().get_matches_from(itr);
        let path = match matches.get_one::<PathBuf>("config_path") {
            Some(config_path) => config_path.to_owned(),
            None => Self::default_path(),
        };
        println!("Path: {path:?}");

        let t = Layer::Toml(path);
        // let e = Layer::Env(Some(env!("CARGO_PKG_NAME").replace("-", "_").to_uppercase()));
        // TODO how this is works?
        let e = Layer::Env(None);
        let c = Layer::Clap(matches);
        let config = Config::with_layers(&[t, e, c]).map_err(|e| crate::error::TwelfError(e))?;
        println!("{config:#?}");

        Ok(config)
    }

    /// Loads the default configuration settings.
    ///
    /// This function first attempts to load the configuration file, and if it fails, it returns the default configuration.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded configuration (`Ok`) or an error (`Err`) if loading fails.
    pub fn load(path: &PathBuf) -> ToDoRes<Self> {
        log::info!("Loading config from: {path:?}");
        Ok(Self::load_from_buffer(
            File::open(path).map_err(|e| ToDoIoError::new(path, e))?,
        )?)
    }

    fn default_path() -> PathBuf {
        const CONFIG_FOLDER: &str = "/.config/";
        const CONFIG_NAME: &str = "todotxt-tui.toml";
        match var("XDG_CONFIG_HOME") {
            Ok(config_path) => PathBuf::from(config_path),
            Err(_) => PathBuf::from(var("HOME").unwrap_or(String::from("~"))).join(CONFIG_FOLDER),
        }
        .join(CONFIG_NAME)
    }

    /// Returns the default configuration file path based on environment variables.
    ///
    /// The configuration file path is determined based on the XDG_CONFIG_HOME and HOME environment variables.
    ///
    /// # Returns
    ///
    /// A `Result` containing the default configuration file path (`Ok`) or an error (`Err`) if the path cannot be determined.
    pub fn load_default() -> ToDoRes<Self> {
        Self::load(&Self::default_path())
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
    pub fn load_from_buffer<R>(mut reader: R) -> ToDoRes<Self>
    where
        R: Read,
    {
        let mut buf = String::default();
        reader.read_to_string(&mut buf).map_err(|e| IOError(e))?;
        Ok(toml::from_str(buf.as_str())?)
    }

    pub fn generate_autocomplete(path: &Path) -> ToDoRes<()> {
        generate(
            Bash,
            &mut Self::command(),
            env!("CARGO_PKG_NAME"),
            &mut File::create(path).map_err(|e| ToDoIoError::new(path, e))?,
        );
        Ok(())
    }

    pub fn export_config(&self, path: &Path) -> ToDoRes<()> {
        let mut output = File::create(path).map_err(|e| ToDoIoError::new(path, e))?;
        write!(output, "{}", toml::to_string_pretty(self)?).map_err(|e| IOError(e))?;
        Ok(())
    }

    pub fn export_default_config(path: &Path) -> ToDoRes<()> {
        let mut output = File::create(path).map_err(|e| ToDoIoError::new(path, e))?;
        write!(output, "{}", toml::to_string_pretty(&Config::default())?)
            .map_err(|e| IOError(e))?;
        Ok(())
    }

    // pub fn get_init_widget(&self) -> WidgetType {
    //     self.init_widget.unwrap_or(WidgetType::List)
    // }
}

fn cli_help_style() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(AnsiColor::Green.on_default().bold())
        .literal(AnsiColor::Cyan.on_default().bold())
        .header(AnsiColor::Green.on_default().bold())
        .invalid(AnsiColor::Yellow.on_default())
        .error(AnsiColor::Red.on_default().bold())
        .valid(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Cyan.on_default())
}

#[cfg(test)]
mod tests {
    use self::parsers::*;
    use super::*;
    use crate::layout::widget::widget_type::WidgetType;
    use std::time::Duration;
    use test_log::test;
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

        assert_eq!(*deserialized.styles.active_color, Color::Green);
        assert_eq!(deserialized.ui_config.init_widget, WidgetType::Done);
        assert_eq!(
            deserialized.ui_config.window_title,
            UiConfig::default().window_title
        );
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
    fn test_load() -> ToDoRes<()> {
        let s = r#"
        active_color = "Blue"
        window_title = "Title"
        todo_path = "path to todo file"
        "#;

        let default = Config::default();
        let c = Config::load_from_buffer(s.as_bytes())?;
        assert_eq!(*c.styles.active_color, Color::Blue);
        assert_eq!(c.ui_config.init_widget, default.ui_config.init_widget);
        assert_eq!(c.ui_config.window_title, String::from("Title"));
        assert_eq!(
            c.file_worker_config.todo_path,
            PathBuf::from("path to todo file")
        );
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
}
