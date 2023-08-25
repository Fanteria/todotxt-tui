use crate::CONFIG;
use super::{
    state_categories::StateCategories, state_list::StateList, state_preview::StatePreview,
    widget_state::WidgetState,
};
use tui::{
    style::Style,
    widgets::{Block, BorderType, Borders},
};

use crossterm::event::KeyEvent;
use tui::{backend::Backend, prelude::Rect, Frame};

#[enum_dispatch]
pub trait State {
    fn handle_key(&mut self, event: &KeyEvent);
    fn render<B: Backend>(&self, f: &mut Frame<B>);
    fn update_chunk(&mut self, chunk: Rect);
    fn get_focus_mut(&mut self) -> &mut bool;
    fn get_focus(&self) -> bool;
    fn get_title(&self) -> &str;

    fn focus(&mut self) {
        *self.get_focus_mut() = true;
    }

    fn unfocus(&mut self) {
        *self.get_focus_mut() = false;
    }

    fn get_block(&self) -> Block {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .title(self.get_title())
            .border_type(BorderType::Rounded);
        if self.get_focus() {
            block = block.border_style(Style::default().fg(CONFIG.active_color));
        }
        block
    }
}
