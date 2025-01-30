use super::widget_type::WidgetType;
use crate::{
    config::{Config, WidgetBorderType},
    ui::EventHandlerUI,
};
use tui::{prelude::Rect, style::Color};

/// Represents the base properties shared among different widget types.
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
    ///
    /// # Parameters
    ///
    /// - `widget_type`: The type of widget.
    /// - `data`: A reference-counted mutex for the `ToDo` data.
    ///
    /// # Returns
    ///
    /// A new `WidgetBase` instance.
    pub fn new(widget_type: &WidgetType, config: &Config) -> Self {
        let event_handler = match widget_type {
            WidgetType::List => config.widget_base_config.tasks_keybind.clone(),
            WidgetType::Done => config.widget_base_config.tasks_keybind.clone(),
            WidgetType::Project => config.widget_base_config.category_keybind.clone(),
            WidgetType::Context => config.widget_base_config.category_keybind.clone(),
            WidgetType::Hashtag => config.widget_base_config.category_keybind.clone(),
            WidgetType::Preview => EventHandlerUI::default(),
        };
        Self {
            title: widget_type.to_string(),
            active_color: *config.styles.active_color,
            focus: false,
            chunk: Rect::default(),
            event_handler,
            border_type: config.widget_base_config.border_type,
        }
    }
}
