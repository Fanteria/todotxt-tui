use super::{widget_list::WidgetList, widget_state::RCToDo, widget_trait::State, Widget};
use crate::todo::ToDoCategory;
use crate::utils::get_block;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    style::{Color, Style},
    widgets::List,
    Frame,
};

pub struct StateCategories {
    state: WidgetList,
    category: ToDoCategory,
    data: RCToDo,
    focus: bool,
}

impl StateCategories {
    pub fn new(category: ToDoCategory, data: RCToDo) -> Self {
        Self {
            state: WidgetList::default(),
            category,
            data,
            focus: false,
        }
    }

    pub fn len(&self) -> usize {
        self.data
            .lock()
            .unwrap()
            .get_categories(self.category)
            .len()
    }
}

impl State for StateCategories {
    fn handle_key(&mut self, event: &KeyEvent) {
        if !self.state.handle_key(event, self.len()) && event.code == KeyCode::Enter {
            let name;
            {
                let todo = self.data.lock().unwrap();
                name = todo
                    .get_categories(self.category)
                    .get_name(self.state.act())
                    .clone();
            }
            self.data
                .lock()
                .unwrap()
                .toggle_filter(self.category, &name);
        }
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>, _: bool, widget: &Widget) {
        let todo = self.data.lock().unwrap();
        let data = todo.get_categories(self.category);
        let list = List::new(data).block(get_block(&widget.title, self.focus));
        if !self.focus {
            f.render_widget(list, widget.chunk)
        } else {
            let list = list.highlight_style(Style::default().bg(Color::LightRed)); // TODO add to config
            f.render_stateful_widget(list, widget.chunk, &mut self.state.state());
        }
    }

    fn focus(&mut self) {
        self.focus = true;
    }

    fn unfocus(&mut self) {
        self.focus = false;
    }
}
