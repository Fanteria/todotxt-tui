use super::widget_state::State;
use super::widget_state::WidgetState;
use super::widget_type::WidgetType;
use crate::todo::ToDo;
use crossterm::event::KeyEvent;
use std::rc::Rc;
use tui::{backend::Backend, layout::Rect, Frame};

pub struct Widget {
    pub widget_type: WidgetType,
    pub chunk: Rect,
    pub title: String,
    state: WidgetState,
}

impl Widget {
    pub fn new(widget_type: WidgetType, title: &str, data: Rc<ToDo>) -> Widget {
        Widget {
            widget_type,
            chunk: Rect {
                width: 0,
                height: 0,
                x: 0,
                y: 0,
            },
            title: title.to_string(),
            state: WidgetState::new(&widget_type, data),
        }
    }

    pub fn update_chunk(&mut self, chunk: Rect) {
        self.chunk = chunk;
    }

    pub fn handle_key(&mut self, event: &KeyEvent) {
        self.state.handle_key(event);
    }

    pub fn focus(&mut self) {
        self.state.focus();
    }

    pub fn unfocus(&mut self) {
        self.state.unfocus();
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, active: bool) {
        self.state.render(f, active, self);
    }
}
