use super::{widget::Widget, widget_type::WidgetType};
use crate::{todo::ToDo, CONFIG};
use crossterm::event::{KeyCode, KeyEvent};
use std::rc::Rc;
use std::cell::RefCell;
use tui::{
    backend::Backend,
    style::Style,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::utils::some_or_return;

type RCToDo = Rc<RefCell<ToDo>>;

#[enum_dispatch]
pub trait State {
    fn handle_key(&mut self, event: &KeyEvent);
    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, widget: &Widget);
    fn focus(&mut self);
    fn unfocus(&mut self);
    fn cursor_visible(&self) -> bool;
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
    data: RCToDo,
    focus: bool,
}

impl StateList {
    fn new(f: fn(&ToDo) -> Vec<ListItem>, data: RCToDo) -> Self {
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
                if (self.f)(&*self.data.borrow()).len() > act {
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
            KeyCode::Char('x') => {
                match self.state.selected() {
                    Some(i) => self.data.borrow_mut().remove_pending_task(i),
                    None => {}
                }
                // TODO panic if there are no tasks
            }
            KeyCode::Char('d') => {
                match self.state.selected() {
                    Some(i) => self.data.borrow_mut().finish_task(i),
                    None => {}
                }
                // TODO panic if there are no tasks
            }
            _ => {}
        }
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, widget: &Widget) {
        let todo = self.data.borrow();
        let data = (self.f)(&*todo);
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

    fn cursor_visible(&self) -> bool {
        return false;
    }
}

pub struct StateInput {
    actual: String,
    data: RCToDo,
}

impl StateInput {
    fn new(data: RCToDo) -> Self {
        Self {
            actual: String::from(""),
            data,
        }
    }

    fn autocomplete(&mut self) {
        let last_space_index = self
            .actual
            .rfind(' ')
            .and_then(|i| Some(i + 1))
            .unwrap_or(0);
        let base = some_or_return!(self.actual.get(last_space_index..));
        let category = some_or_return!(base.get(0..1));
        let pattern = some_or_return!(base.get(1..));

        let get_list = || match category {
            "+" => Some(self.data.borrow().get_projects()),
            "@" => Some(self.data.borrow().get_contexts()),
            "#" => Some(self.data.borrow().get_hashtags()),
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
        if list.is_empty() {
            return;
        }

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

    fn cursor_visible(&self) -> bool {
        return true;
    }
}

#[enum_dispatch(State)]
pub enum WidgetState {
    Input(StateInput),
    List(StateList),
}

impl WidgetState {
    pub fn new(widget_type: &WidgetType, data: Rc<RefCell<ToDo>>) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_autocomplete() -> Result<(), Box<dyn Error>> {
        // prepare testing
        let testing_string: &str = r#"
        1 +project1 @context1 #hashtag1 
        2 +project2 @context2
        3 +project3 @context3
        4 +name_project2 @context3 #hashtag1
        5 +name_project3 @context3 #hashtag2
        6 +unique @context2 #hashtag2
        "#;
        let mut widget = StateInput::new(Rc::new(RefCell::new(ToDo::load(testing_string.as_bytes(), false)?)));

        // not found check
        widget.actual = String::from("some text +missing");
        widget.autocomplete();
        assert_eq!(widget.actual, "some text +missing");

        // group check
        widget.actual = String::from("some text +pr");
        widget.autocomplete();
        assert_eq!(widget.actual, "some text +project");

        // double group check
        widget.actual = String::from("some text +project1 +name");
        widget.autocomplete();
        assert_eq!(widget.actual, "some text +project1 +name_project");

        // unique check
        widget.actual = String::from("text +uni");
        widget.autocomplete();
        assert_eq!(widget.actual, "text +unique ");

        // empty task description check
        widget.actual = String::from("+uni");
        widget.autocomplete();
        assert_eq!(widget.actual, "+unique ");

        // context check
        widget.actual = String::from("@con");
        widget.autocomplete();
        assert_eq!(widget.actual, "@context");

        widget.actual = String::from("@context1");
        widget.autocomplete();
        assert_eq!(widget.actual, "@context1 ");

        // hashtag check
        widget.actual = String::from("#hash");
        widget.autocomplete();
        assert_eq!(widget.actual, "#hashtag");

        widget.actual = String::from("#hashtag2");
        widget.autocomplete();
        assert_eq!(widget.actual, "#hashtag2 ");

        Ok(())
    }
}
