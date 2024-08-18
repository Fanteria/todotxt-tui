mod colors;
mod conf;
mod configs;
mod keycode;
mod parsers;
mod styles;
mod text_modifier;
mod text_style;

pub use self::colors::Color;
pub use self::keycode::KeyCodeDef;
pub use self::styles::CustomCategoryStyle;
pub use self::styles::StylesValue;
pub use self::text_style::TextStyle;
pub use self::text_style::TextStyleList;

pub use self::conf::Conf;
pub use self::conf::ConfMerge;
pub use self::conf::ConfigDefaults;

pub use self::configs::*;

#[cfg(test)]
mod tests {
    use self::parsers::*;
    use super::*;
    use crate::{layout::widget::widget_type::WidgetType, ToDoRes};
    use std::{path::PathBuf, time::Duration};
    use test_log::test;
    use tui::style::Color;

    #[test]
    fn test_deserialization() {
        let deserialized = Config::from_reader(
            r#"
            active_color = "Green"
            init_widget = "Done"
        "#
            .as_bytes(),
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
    fn test_load() -> ToDoRes<()> {
        let s = r#"
        active_color = "Blue"
        window_title = "Title"
        todo_path = "path to todo file"
        "#;

        let default = Config::default();
        let c = Config::from_reader(s.as_bytes())?;
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
    fn help_can_be_generated() -> ToDoRes<()> {
        Config::from_args(Vec::<&str>::new())?;
        Ok(())
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("1000"), Ok(Duration::from_secs(1000)));
        assert!(parse_duration("-1000").is_err());
    }
}
