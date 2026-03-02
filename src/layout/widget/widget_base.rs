use super::widget_type::WidgetType;
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
    pub fn new(widget_type: &WidgetType, config: &Config) -> Self {
        let c = &config.widget_base_config;
        let (event_handler, title) = match widget_type {
            WidgetType::List => (c.tasks_keybind.clone(), c.pending_widget_name.clone()),
            WidgetType::Done => (c.tasks_keybind.clone(), c.done_widget_name.clone()),
            WidgetType::Project => (c.category_keybind.clone(), c.project_widget_name.clone()),
            WidgetType::Context => (c.category_keybind.clone(), c.context_widget_name.clone()),
            WidgetType::Hashtag => (c.category_keybind.clone(), c.hashtag_widget_name.clone()),
            WidgetType::Preview => (EventHandlerUI::default(), c.preview_widget_name.clone()),
            WidgetType::PendingLivePreview => (
                EventHandlerUI::default(),
                c.pending_live_preview_widget_name.clone(),
            ),
            WidgetType::DoneLivePreview => (
                EventHandlerUI::default(),
                c.done_live_preview_widget_name.clone(),
            ),
        };
        Self {
            title,
            active_color: *config.styles.active_color,
            focus: false,
            chunk: Rect::default(),
            event_handler,
            border_type: config.widget_base_config.border_type,
        }
    }
}
