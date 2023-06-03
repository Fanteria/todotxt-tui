use super::{widget::Widget, widget_type::WidgetType};
use crate::{
    todo::{TaskList, ToDo},
    CONFIG,
};
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

#[enum_dispatch]
pub trait State {
    fn handle_key(&mut self, event: &KeyEvent);
    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, widget: &Widget);
    fn focus(&mut self);
    fn unfocus(&mut self);
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
    f: fn(&ToDo) -> Vec<ListItem>,
    focus: bool,
}

impl StateList {
    fn new(f: fn(&ToDo) -> Vec<ListItem>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            state,
            f,
            focus: false,
        }
    }
}

impl State for StateList {
    fn handle_key(&mut self, event: &KeyEvent) {
        match event.code {
            KeyCode::Char('j') => {
                self.state
                    .select(self.state.selected().and_then(|i| Some(i + 1)));
            }
            KeyCode::Char('k') => {
                self.state
                    .select(self.state.selected().and_then(|i| Some(i - 1)));
            }
            _ => {}
        }
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, widget: &Widget) {
        let data = (self.f)(&widget.data);
        let list = List::new(data.clone())
            .block(get_block(&widget.title, active))
            // .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        f.render_stateful_widget(list, widget.chunk, &mut self.state.clone());
    }

    fn focus(&mut self) {
        self.focus = true;
    }

    fn unfocus(&mut self) {
        self.focus = false;
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
            KeyCode::Backspace => {
                self.actual.pop();
            }
            KeyCode::Esc => self.actual.clear(),
            _ => {}
        }
    }
    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, widget: &Widget) {
        f.render_widget(
            Paragraph::new(self.actual.clone()).block(get_block(&widget.title, active)),
            widget.chunk,
        );
    }

    fn focus(&mut self) {}

    fn unfocus(&mut self) {}
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
            WidgetType::List => WidgetState::List(StateList::new(|todo| {
                Into::<Vec<ListItem>>::into(todo.pending.clone())
            })),
            WidgetType::Done => WidgetState::List(StateList::new(|todo| { 
                Into::<Vec<ListItem>>::into(todo.done.clone())
            })),
            WidgetType::Project => WidgetState::List(StateList::new(|todo| { 
                Into::<Vec<ListItem>>::into(todo.get_projects())
            })),
            WidgetType::Context => WidgetState::List(StateList::new(|todo| { 
                Into::<Vec<ListItem>>::into(todo.get_contexts())
            })),
        }
    }
}
