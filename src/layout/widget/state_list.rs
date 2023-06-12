use super::{widget_trait::State, Widget};
use crate::todo::ToDo;
use crate::utils::get_block;
use crossterm::event::{KeyCode, KeyEvent};
use std::cell::RefCell;
use std::rc::Rc;
use tui::{
    backend::Backend,
    widgets::{List, ListItem, ListState},
    Frame,
};

type RCToDo = Rc<RefCell<ToDo>>;

pub struct StateList {
    state: ListState,
    f: fn(&ToDo) -> Vec<ListItem>,
    data: RCToDo,
    focus: bool,
}

impl StateList {
    pub fn new(f: fn(&ToDo) -> Vec<ListItem>, data: RCToDo) -> Self {
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
