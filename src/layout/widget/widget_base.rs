use crate::{
    config::{Config, WidgetBorderType},
    ui::EventHandlerUI,
};
use tui::{prelude::Rect, style::Color};

/// Represents the base properties shared among different widget types.
#[derive(Debug)]
pub struct WidgetBase {
    pub title: String,
    pub active_color: Color,
    pub focus: bool,
    pub chunk: Rect,
    pub event_handler: EventHandlerUI,
    pub border_type: WidgetBorderType,
}

impl WidgetBase {
    /// Creates a new `WidgetBase` instance for a specific widget type.
    pub fn new(title: impl Into<String>, config: &Config) -> Self {
        Self {
            title: title.into(),
            active_color: *config.styles.active_color,
            focus: false,
            chunk: Rect::default(),
            event_handler: EventHandlerUI::default(),
            border_type: config.widget_base_config.border_type,
        }
    }

    pub fn events(mut self, events: EventHandlerUI) -> Self {
        self.event_handler = events;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn events_builder_sets_handler_and_title() {
        let config = Config::default();
        let handler = config.widget_base_config.category_keybind.clone();
        let base = WidgetBase::new("my widget", &config).events(handler.clone());
        assert_eq!(base.title, "my widget");
        assert_eq!(base.event_handler, handler);
    }
}
