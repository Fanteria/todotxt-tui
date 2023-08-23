use super::{
    state_categories::StateCategories, state_list::StateList,
    state_preview::StatePreview, widget_state::WidgetState, Widget,
};
use crossterm::event::KeyEvent;
use tui::{backend::Backend, Frame};

#[enum_dispatch]
pub trait State {
    fn handle_key(&mut self, event: &KeyEvent);
    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, widget: &Widget);
    fn focus(&mut self);
    fn unfocus(&mut self);
}

pub trait List {

}
