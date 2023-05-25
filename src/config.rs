use crate::layout::widget::WidgetType;
use serde::{Deserialize, Serialize};
use std::env::{var, VarError};
use std::error::Error;
use std::fs;
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
}

impl Config {
    pub fn default() -> Self {
        Self {
            init_widget: Self::default_widget_type(),
            active_color: Self::default_color(),
            window_title: Self::default_window_title(),
        }
    }

    pub fn load_default() -> Self {
        if let Ok(path) = &Self::default_path() {
            return Self::load_config(path);
        }
        Self::default()
    }

    pub fn load_config(path: &str) -> Self {
        let read_from_file = |path: &str| -> Result<Self, Box<dyn Error>> {
            let ret: Self = toml::from_str(&fs::read_to_string(path)?)?;
            Ok(ret)
        };
        read_from_file(path).unwrap_or(Self::default())
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
    use std::fs;
    use std::fs::OpenOptions;
    use std::io::Result;
    use std::io::Write;
    use std::path::Path;
    use tui::style::Color;

    fn test_path(filename: &str) -> String {
        String::from(env!("CARGO_MANIFEST_DIR"))
            + "/resources/test/tmp/"
            + "config_test/"
            + filename
            + ".conf"
    }

    fn write_to_test_file(filename: &str, content: &str) -> Result<()> {
        if Path::new(&test_path(filename)).exists() {
            fs::remove_file(test_path(filename))?;
        }
        let path_string = test_path(filename);
        let path = Path::new(&path_string);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();

        let mut f = OpenOptions::new()
            .write(true)
            .append(false)
            .create(true)
            .open(path_string)?;
        f.write(content.as_bytes())?;
        Ok(())
    }

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
        write_to_test_file(
            "test_load",
            r#"
        active_color = "Blue"
        window_title = "Title"
        "#,
        )?;

        let c = Config::load_config(&test_path("test_load"));
        assert_eq!(c.active_color, Color::Blue);
        assert_eq!(c.init_widget, WidgetType::List);
        assert_eq!(c.window_title, "Title");
        Ok(())
    }

    #[test]
    fn test_default() -> Result<()> {
        write_to_test_file(
            "test_default",
            r#"

        "#,
        )?;
        assert_eq!(
            Config::load_config(&test_path("test_default")),
            Config::default()
        );

        fs::remove_file(test_path("test_default"))?;
        assert_eq!(
            Config::load_config(&test_path("test_default")),
            Config::default()
        );

        Ok(())
    }
}
