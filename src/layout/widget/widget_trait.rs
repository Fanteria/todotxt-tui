use super::widget_base::WidgetBase;
use crate::todo::ToDo;
use crate::CONFIG;
use crossterm::event::KeyEvent;
use std::sync::MutexGuard;
use tui::{backend::Backend, prelude::Rect, Frame};
use tui::{
    style::Style,
    widgets::{Block, BorderType, Borders},
};

#[enum_dispatch]
pub trait State {
    fn handle_key(&mut self, event: &KeyEvent);
    fn render<B: Backend>(&self, f: &mut Frame<B>);
    fn get_base(&self) -> &WidgetBase;
    fn get_base_mut(&mut self) -> &mut WidgetBase;

    fn data(&self) -> MutexGuard<'_, ToDo> {
        self.get_base().data.lock().unwrap()
    }

    fn focus(&mut self) {
        self.get_base_mut().focus = true;
    }

    fn unfocus(&mut self) {
        self.get_base_mut().focus = false;
    }

    fn update_chunk(&mut self, chunk: Rect) {
        self.get_base_mut().chunk = chunk;
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
}
