use super::{
    widget_base::WidgetBase, widget_list::WidgetList, widget_state::RCToDo, widget_trait::State,
};
use crate::todo::ToDoCategory;
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
    base: WidgetBase,
}

impl StateCategories {
    pub fn new(category: ToDoCategory, data: RCToDo, title: &str) -> Self {
        Self {
            state: WidgetList::default(),
            category,
            data: data.clone(),
            base: WidgetBase::new(title, data),
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

    fn render<B: Backend>(&self, f: &mut Frame<B>) {
        let todo = self.data.lock().unwrap();
        let data = todo.get_categories(self.category);
        let list = List::new(data).block(self.get_block());
        if !self.base.focus {
            f.render_widget(list, self.base.chunk)
        } else {
            let list = list.highlight_style(Style::default().bg(Color::LightRed)); // TODO add to config
            f.render_stateful_widget(list, self.base.chunk, &mut self.state.state());
        }
    }

    fn get_base(&self) -> &WidgetBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut WidgetBase {
        &mut self.base
    }
}
