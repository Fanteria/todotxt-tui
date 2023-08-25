use super::{widget_list::WidgetList, widget_state::RCToDo, widget_trait::State};
use crate::todo::ToDoCategory;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    prelude::Rect,
    style::{Color, Style},
    widgets::List,
    Frame,
};

pub struct StateCategories {
    state: WidgetList,
    category: ToDoCategory,
    data: RCToDo,
    focus: bool,
    chunk: Rect,
    title: String,
}

impl StateCategories {
    pub fn new(category: ToDoCategory, data: RCToDo, title: &str) -> Self {
        Self {
            state: WidgetList::default(),
            category,
            data,
            focus: false,
            chunk: Rect::default(),
            title: String::from(title),
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
        if !self.focus {
            f.render_widget(list, self.chunk)
        } else {
            let list = list.highlight_style(Style::default().bg(Color::LightRed)); // TODO add to config
            f.render_stateful_widget(list, self.chunk, &mut self.state.state());
        }
    }

    fn update_chunk(&mut self, chunk: Rect) {
        self.chunk = chunk;
    }

    fn get_focus_mut(&mut self) -> &mut bool {
        &mut self.focus
    }

    fn get_focus(&self) -> bool {
        self.focus
    }

    fn get_title(&self) -> &str {
        &self.title
    }
}
