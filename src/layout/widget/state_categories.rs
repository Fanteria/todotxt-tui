use super::{widget_state::RCToDo, widget_trait::State, Widget};
use crate::todo::ToDoCategory;
use crate::utils::get_block;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    widgets::{List, ListState},
    Frame,
};

pub struct StateCategories {
    state: ListState,
    category: ToDoCategory,
    data: RCToDo,
    focus: bool,
}

impl StateCategories {
    pub fn new(
        category: ToDoCategory,
        data: RCToDo,
    ) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            state,
            category,
            data,
            focus: false,
        }
    }

    pub fn len(&self) -> usize {
        self.data.lock().unwrap().get_category(self.category).len()
    }

    pub fn act(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }
}

impl State for StateCategories {
    fn handle_key(&mut self, event: &KeyEvent) {
        match event.code {
            KeyCode::Char('j') => {
                let act = self.act() + 1;
                if self.len() > act {
                    self.state.select(Some(act));
                }
            }
            KeyCode::Char('k') => {
                let act = self.act();
                if 0 < act {
                    self.state.select(Some(act - 1));
                }
            }
            KeyCode::Enter => {
                let name;
                {
                    let todo = self.data.lock().unwrap();
                    name = todo.get_category(self.category).get_name(self.act()).clone();
                }
                self.data.lock().unwrap().toggle_filter(self.category, &name);
            }
            _ => {}
        }
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>, _: bool, widget: &Widget) {
        let todo = self.data.lock().unwrap();
        let data = todo.get_category(self.category);
        let list = List::new(data).block(get_block(&widget.title, self.focus));
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
