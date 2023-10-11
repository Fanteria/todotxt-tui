use super::{widget_type::WidgetType, RCToDo};
use crate::todo::ToDo;
use crate::{ui::EventHandlerUI, CONFIG};
use std::sync::MutexGuard;
use tui::prelude::Rect;

/// Represents the base properties shared among different widget types.
pub struct WidgetBase {
    pub title: String,
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
    pub fn new(widget_type: &WidgetType, data: RCToDo) -> Self {
        let event_handler = match widget_type {
            WidgetType::List => CONFIG.tasks_keybind.clone(),
            WidgetType::Done => CONFIG.tasks_keybind.clone(),
            WidgetType::Project => CONFIG.category_keybind.clone(),
            WidgetType::Context => CONFIG.category_keybind.clone(),
            WidgetType::Hashtag => CONFIG.category_keybind.clone(),
            WidgetType::Preview => EventHandlerUI::default(),
        };
        Self {
            title: widget_type.to_string(),
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
