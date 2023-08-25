mod state_categories;
mod state_list;
mod state_preview;
mod widget_state;
mod widget_trait;
mod widget_list;
mod widget_base;
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
    state: WidgetState,
}

impl Widget {
    pub fn new(widget_type: WidgetType, data: Arc<Mutex<ToDo>>) -> Widget {
        Widget {
            widget_type,
            state: WidgetState::new(&widget_type, data),
        }
    }

    pub fn update_chunk(&mut self, chunk: Rect) {
        self.state.update_chunk(chunk);
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

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, _: bool) {
        self.state.render(f);
    }
}
