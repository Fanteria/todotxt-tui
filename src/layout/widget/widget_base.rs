use super::RCToDo;
use tui::prelude::Rect;

pub struct WidgetBase {
    pub title: String,
    pub focus: bool,
    pub chunk: Rect,
    pub data: RCToDo,
}

impl WidgetBase {
    pub fn new(title: String, data: RCToDo) -> Self {
        Self {
            title,
            focus: false,
            chunk: Rect::default(),
            data,
        }
    }
}