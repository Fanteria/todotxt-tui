use super::widget_type::WidgetType;
use crate::CONFIG;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders, List, ListState, Paragraph},
    Frame,
};

#[enum_dispatch]
pub trait State {
    fn handle_key(&mut self, event: &KeyEvent);
    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, title: &str, area: Rect);
}

fn get_block(title: &str, active: bool) -> Block {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(title.clone())
        .border_type(BorderType::Rounded);
    if active {
        block = block.border_style(Style::default().fg(CONFIG.active_color));
    }
    block
}

#[allow(dead_code)]
pub struct StateList {
    state: ListState,
}

impl StateList {
    fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self { state }
    }
}

impl State for StateList {
    fn handle_key(&mut self, _event: &KeyEvent) {}
    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, title: &str, area: Rect) {
        f.render_widget(get_block(title, active), area);
    }
}

pub struct StateInput {
    actual: String,
}

impl StateInput {
    fn new() -> Self {
        Self {
            actual: String::from(""),
        }
    }
}

impl State for StateInput {
    fn handle_key(&mut self, event: &KeyEvent) {
        match event.code {
            KeyCode::Char(ch) => self.actual.push(ch),
            _ => {}
        }
    }
    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, title: &str, area: Rect) {
        f.render_widget(Paragraph::new(self.actual.clone()).block(get_block(title, active)), area);
    }
}

#[enum_dispatch(State)]
pub enum WidgetState {
    Input(StateInput),
    List(StateList),
}

impl WidgetState {
    pub fn new(widget_type: &WidgetType) -> Self {
        match widget_type {
            WidgetType::Input => WidgetState::Input(StateInput::new()),
            _ => WidgetState::List(StateList::new()),
        }
    }
}
