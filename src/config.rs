use crate::layout::widget::WidgetType;
use serde::{Deserialize, Serialize};
use tui::style::Color;

#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize, PartialEq, Debug))]
pub struct Config {
    #[serde(with = "ColorDef", default = "Config::default_color")]
    active_color: Color,
    #[serde(default = "Config::default_widget_type")]
    init_widget: WidgetType,
}

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

impl Config {
    pub fn default() -> Config {
        Config {
            init_widget: Config::default_widget_type(),
            active_color: Config::default_color(),
        }
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
    use tui::style::Color;
    use crate::layout::widget::WidgetType;


    #[test]
    fn test_deserialization() {
        let deserialized: Config = toml::from_str(r#"
            active_color = "Green"
            init_widget = "Done"
        "#).unwrap(); 

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
