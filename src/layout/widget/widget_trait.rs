use super::{
    state_categories::StateCategories, state_list::StateList,
    state_preview::StatePreview, widget_state::WidgetState, Widget,
};
use crossterm::event::KeyEvent;
use tui::{backend::Backend, Frame, prelude::Rect};

#[enum_dispatch]
pub trait State {
    fn handle_key(&mut self, event: &KeyEvent);
    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, widget: &Widget);
    fn update_chunk(&mut self, chunk: Rect);
    fn get_focus(&mut self) -> &mut bool;
    fn focus(&mut self) {
        *self.get_focus() = true;
    }

    fn unfocus(&mut self) {
        *self.get_focus() = false;
    }
}
