mod state_categories;
mod state_input;
mod state_list;
mod state_preview;
mod widget_state;
mod widget_trait;
pub mod widget_type;

use crate::todo::ToDo;
use crossterm::event::KeyEvent;
use std::sync::{Arc, Mutex};
use tui::{backend::Backend, layout::Rect, Frame};
use widget_state::WidgetState;
use widget_trait::State;
use widget_type::WidgetType;

pub struct Widget {
    pub widget_type: WidgetType,
    pub chunk: Rect,
    pub title: String,
    state: WidgetState,
}

impl Widget {
    pub fn new(widget_type: WidgetType, title: &str, data: Arc<Mutex<ToDo>>) -> Widget {
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

    pub fn cursor_visible(&self) -> bool {
        self.state.cursor_visible()
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, active: bool) {
        self.state.render(f, active, self);
    }
}
