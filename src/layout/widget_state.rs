use super::{widget::Widget, widget_type::WidgetType};
use crate::{todo::ToDo, CONFIG};
use crossterm::event::{KeyCode, KeyEvent};
use std::rc::Rc;
use tui::{
    backend::Backend,
    style::Style,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::utils::some_or_return;

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

pub struct StateList {
    state: ListState,
    f: fn(&ToDo) -> Vec<ListItem>,
    data: Rc<ToDo>,
    focus: bool,
}

impl StateList {
    fn new(f: fn(&ToDo) -> Vec<ListItem>, data: Rc<ToDo>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            state,
            f,
            data,
            focus: false,
        }
    }
}

impl State for StateList {
    fn handle_key(&mut self, event: &KeyEvent) {
        match event.code {
            KeyCode::Char('j') => {
                let act = match self.state.selected() {
                    Some(a) => a + 1,
                    None => 0,
                };
                if (self.f)(&self.data).len() > act {
                    self.state.select(Some(act));
                }
            }
            KeyCode::Char('k') => {
                let act = match self.state.selected() {
                    Some(a) => a,
                    None => 0,
                };
                if 0 < act {
                    self.state.select(Some(act - 1));
                }
            }
            _ => {}
        }
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, widget: &Widget) {
        let data = (self.f)(&self.data);
        let list = List::new(data.clone()).block(get_block(&widget.title, active));
        if !self.focus {
            f.render_widget(list, widget.chunk)
        } else {
            // .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            let list = list.highlight_symbol(">>");
            f.render_stateful_widget(list, widget.chunk, &mut self.state.clone());
        }
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
    data: Rc<ToDo>,
}

impl StateInput {
    fn new(data: Rc<ToDo>) -> Self {
        Self {
            actual: String::from(""),
            data,
        }
    }

    fn autocomplete(&mut self) {
        let last_space_index = some_or_return!(self.actual.rfind(' '));
        let base = some_or_return!(self.actual.get(last_space_index + 1..));
        let category = some_or_return!(base.get(0..1));
        let pattern = some_or_return!(base.get(1..));

        let get_list = || match category {
            "+" => Some(self.data.get_projects()),
            "@" => Some(self.data.get_contexts()),
            "#" => Some(self.data.get_hashtags()),
            _ => None,
        };

        let list = some_or_return!(get_list());
        if list.is_empty() {
            return;
        }

        let list = list.start_with(pattern);

        let same_start_index = |fst: &str, sec: &str| -> usize {
            for (i, (fst_char, sec_char)) in fst
                .chars()
                .into_iter()
                .zip(sec.chars().into_iter())
                .enumerate()
            {
                if fst_char != sec_char {
                    return i;
                }
            }
            std::cmp::min(fst.len(), sec.len())
        };

        let mut new_act = list[0].as_str();

        if list.len() != 1 {
            list.iter()
                .skip(1)
                .for_each(|item| new_act = &new_act[..same_start_index(new_act, item)]);
            self.actual += &new_act[pattern.len()..];
        } else {
            self.actual += &new_act[pattern.len()..];
            self.actual += " ";
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
            KeyCode::Tab => self.autocomplete(),
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
    pub fn new(widget_type: &WidgetType, data: Rc<ToDo>) -> Self {
        match widget_type {
            WidgetType::Input => WidgetState::Input(StateInput::new(data)),
            WidgetType::List => WidgetState::List(StateList::new(
                |todo| Into::<Vec<ListItem>>::into(todo.pending.clone()),
                data,
            )),
            WidgetType::Done => WidgetState::List(StateList::new(
                |todo| Into::<Vec<ListItem>>::into(todo.done.clone()),
                data,
            )),
            WidgetType::Project => WidgetState::List(StateList::new(
                |todo| Into::<Vec<ListItem>>::into(todo.get_projects()),
                data,
            )),
            WidgetType::Context => WidgetState::List(StateList::new(
                |todo| Into::<Vec<ListItem>>::into(todo.get_contexts()),
                data,
            )),
        }
    }
}
