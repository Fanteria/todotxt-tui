use super::widget_state::RCToDo;
use tui::prelude::Rect;

pub struct WidgetBase {
    pub title: String,
    pub focus: bool,
    pub chunk: Rect,
    pub data: RCToDo,
}

impl WidgetBase {
    pub fn new(title: &str, data: RCToDo) -> Self {
        Self {
            title: String::from(title),
            focus: false,
            chunk: Rect::default(),
            data,
        }
    }
}
