use super::super::Render;
use super::widget_base::WidgetBase;
use crate::todo::ToDo;
use crate::ui::{HandleEvent, UIEvent};
use crate::CONFIG;
use crossterm::event::KeyCode;
use std::sync::MutexGuard;
use tui::{
    backend::Backend,
    prelude::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders},
    Frame,
};

#[enum_dispatch]
pub trait State {
    // fn handle_key(&mut self, event: &KeyEvent);
    fn handle_event_state(&mut self, event: UIEvent) -> bool;
    fn render<B: Backend>(&self, f: &mut Frame<B>);
    fn get_base(&self) -> &WidgetBase;
    fn get_base_mut(&mut self) -> &mut WidgetBase;

    fn data(&self) -> MutexGuard<'_, ToDo> {
        self.get_base().data.lock().unwrap()
    }

    fn get_block(&self) -> Block {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .title(self.get_base().title.clone())
            .border_type(BorderType::Rounded);
        if self.get_base().focus {
            block = block.border_style(Style::default().fg(CONFIG.active_color));
        }
        block
    }

    fn focus_event(&mut self) {}
    fn unfocus_event(&mut self) {}
    fn update_chunk_event(&mut self) {}
    fn get_internal_event(&self, _: &KeyCode) -> UIEvent {
        UIEvent::None
    }
}

impl<S: State> HandleEvent for S {
    fn get_event(&self, key: &KeyCode) -> UIEvent {
        let event = self.get_internal_event(key);
        if event == UIEvent::None {
            self.get_base().event_handler.get_event(key)
        } else {
            event
        }
    }

    fn handle_event(&mut self, event: UIEvent) -> bool {
        self.handle_event_state(event)
    }
}

impl<S: State> Render for S {
    fn render<B: Backend>(&self, f: &mut Frame<B>) {
        State::render(self, f);
    }

    fn focus(&mut self) {
        self.get_base_mut().focus = true;
        self.focus_event();
    }

    fn unfocus(&mut self) {
        self.get_base_mut().focus = false;
        self.unfocus_event();
    }

    fn update_chunk(&mut self, chunk: Rect) {
        self.get_base_mut().chunk = chunk;
        self.update_chunk_event();
    }
}
