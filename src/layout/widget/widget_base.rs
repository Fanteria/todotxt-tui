use super::{widget_type::WidgetType, RCToDo};
use crate::{config::Config, todo::ToDo, ui::EventHandlerUI};
use std::sync::MutexGuard;
use tui::{prelude::Rect, style::Color};

/// Represents the base properties shared among different widget types.
pub struct WidgetBase {
    pub title: String,
    pub active_color: Color,
    pub focus: bool,
    pub chunk: Rect,
    pub data: RCToDo,
    pub event_handler: EventHandlerUI,
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
    pub fn new(widget_type: &WidgetType, data: RCToDo, config: &Config) -> Self {
        let event_handler = match widget_type {
            WidgetType::List => config.get_tasks_keybind(),
            WidgetType::Done => config.get_tasks_keybind(),
            WidgetType::Project => config.get_category_keybind(),
            WidgetType::Context => config.get_category_keybind(),
            WidgetType::Hashtag => config.get_category_keybind(),
            WidgetType::Preview => EventHandlerUI::default(),
        };
        Self {
            title: widget_type.to_string(),
            active_color: config.get_active_color(),
            focus: false,
            chunk: Rect::default(),
            data,
            event_handler,
        }
    }

    /// Gets a mutable reference to the `ToDo` data stored in the widget.
    ///
    /// # Returns
    ///
    /// A `MutexGuard` representing a mutable reference to the `ToDo` data.
    pub fn data(&self) -> MutexGuard<'_, ToDo> {
        self.data.lock().unwrap()
    }
}
