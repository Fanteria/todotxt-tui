use crate::layout::widget::WidgetType;
use serde::{Deserialize, Serialize};
use std::env::{var, VarError};
use std::error::Error;
use std::fs;
use tui::style::Color;

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
}

impl Config {
    pub fn default() -> Config {
        Config {
            init_widget: Config::default_widget_type(),
            active_color: Config::default_color(),
        }
    }

    pub fn config_path() -> Result<String, VarError> {
        var("XDG_CONFIG_HOME").or_else(|_| var("HOME").map(|home| format!("{}/.config", home)))
    }

    pub fn load_config() -> Config {
        let read_from_file = |path: &str| -> Result<Config, Box<
        dyn Error>> {
            let ret: Config = toml::from_str(&fs::read_to_string(path)?)?;
            Ok(ret)
        };
        if let Ok(config_path) = &Config::config_path() {
            return read_from_file(config_path).unwrap_or(Config::default());
        }
        Config::default()
    }

    fn default_color() -> Color {
        Color::Red
    }

    fn default_widget_type() -> WidgetType {
        WidgetType::List
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::layout::widget::WidgetType;
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
    }

    #[test]
    fn test_serialization() {
        let c = Config::default();
        let serialized = toml::to_string_pretty(&c).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(c, deserialized);
    }
}
