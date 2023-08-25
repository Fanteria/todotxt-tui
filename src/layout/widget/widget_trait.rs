use crate::CONFIG;
use super::{
    state_categories::StateCategories, state_list::StateList, state_preview::StatePreview,
    widget_state::WidgetState, widget_base::WidgetBase,
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
    fn get_base(&self) -> &WidgetBase;
    fn get_base_mut(&mut self) -> &mut WidgetBase;

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
