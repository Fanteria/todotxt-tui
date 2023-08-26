use tui::{backend::Backend, prelude::Rect, Frame};

pub trait Render {
    fn render<B: Backend>(&self, f: &mut Frame<B>);
    fn focus(&mut self);
    fn unfocus(&mut self);
    fn update_chunk(&mut self, chunk: Rect);
}
