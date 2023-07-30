use super::{widget_state::RCToDo, widget_trait::State, Widget};
use crate::todo::{CategoryList, ToDo};
use crate::utils::get_block;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    widgets::{List, ListState},
    Frame,
};

pub struct StateCategories {
    state: ListState,
    fn_list: fn(&ToDo) -> CategoryList,
    fn_toggle: fn(&mut ToDo, &str),
    data: RCToDo,
    focus: bool,
}

impl StateCategories {
    pub fn new(fn_list: fn(&ToDo) -> CategoryList, fn_toggle: fn(&mut ToDo, &str), data: RCToDo) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            state,
            fn_list,
            fn_toggle,
            data,
            focus: false,
        }
    }
}

impl State for StateCategories {
    fn handle_key(&mut self, event: &KeyEvent) {
        match event.code {
            KeyCode::Char('j') => {
                let act = self.state.selected().unwrap_or(0);               if (self.fn_list)(&self.data.borrow()).len() > act {
                    self.state.select(Some(act));
                }
            }
            KeyCode::Char('k') => {
                let act = self.state.selected().unwrap_or(0);
                if 0 < act {
                    self.state.select(Some(act - 1));
                }
            }
            KeyCode::Enter => {
                if let Some(index) = self.state.selected() {
                    let name;
                    {
                        let todo = self.data.borrow();
                        let data = (self.fn_list)(&todo);
                        name = data.get_name(index).clone();
                    }
                    (self.fn_toggle)(&mut self.data.borrow_mut(), &name)
                }
            }
            _ => {}
        }
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, widget: &Widget) {
        let todo = self.data.borrow();
        let data = (self.fn_list)(&todo);
        let list = List::new(data).block(get_block(&widget.title, active));
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
        false
    }
}