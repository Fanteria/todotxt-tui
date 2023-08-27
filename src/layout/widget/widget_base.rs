use super::{widget_type::WidgetType, RCToDo};
use crate::todo::ToDo;
use crate::{ui::EventHandler, CONFIG};
use std::sync::MutexGuard;
use tui::prelude::Rect;

pub struct WidgetBase {
    pub title: String,
    pub focus: bool,
    pub chunk: Rect,
    pub data: RCToDo,
    pub event_handler: EventHandler,
}

impl WidgetBase {
    pub fn new(widget_type: &WidgetType, data: RCToDo) -> Self {
        let event_handler = match widget_type {
            WidgetType::List => CONFIG.tasks_keybind.clone(),
            WidgetType::Done => CONFIG.tasks_keybind.clone(),
            WidgetType::Project => CONFIG.category_keybind.clone(),
            WidgetType::Context => CONFIG.category_keybind.clone(),
            WidgetType::Hashtag => CONFIG.category_keybind.clone(),
            WidgetType::Preview => EventHandler::default(),
        };
        Self {
            title: widget_type.to_string(),
            focus: false,
            chunk: Rect::default(),
            data,
            event_handler,
        }
    }

    pub fn data(&self) -> MutexGuard<'_, ToDo> {
        self.data.lock().unwrap()
    }
}
