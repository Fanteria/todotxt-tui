use super::{widget_base::WidgetBase, widget_list::WidgetList, widget_trait::State};
use crate::{
    todo::ToDoCategory,
    ui::{HandleEvent, UIEvent},
};
use crossterm::event::KeyCode;
use tui::{
    backend::Backend,
    style::{Color, Style},
    widgets::List,
    Frame,
};

pub struct StateCategories {
    base: WidgetList,
    pub category: ToDoCategory,
}

impl StateCategories {
    pub fn new(base: WidgetList, category: ToDoCategory) -> Self {
        Self {
            base,
            category,
        }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.base.data().get_categories(self.category).len()
    }
}

impl State for StateCategories {
    fn handle_event_state(&mut self, event: UIEvent) -> bool {
        if self.base.handle_event(event) {
            return true;
        }
        match event {
            UIEvent::Select => {
                let name;
                {
                    let todo = self.base.data();
                    name = todo
                        .get_categories(self.category)
                        .get_name(self.base.act())
                        .clone();
                }
                self.base.data().toggle_filter(self.category, &name);
                self.base.len = self.len();
            }
            _ => return false,
        }
        true
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>) {
        let todo = self.base.data();
        let data = todo.get_categories(self.category);
        let list = List::new(data).block(self.get_block());
        if !self.base.focus {
            f.render_widget(list, self.base.chunk)
        } else {
            let list = list.highlight_style(Style::default().bg(Color::LightRed)); // TODO add to config
            f.render_stateful_widget(list, self.base.chunk, &mut self.base.state());
        }
    }

    fn get_base(&self) -> &WidgetBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut WidgetBase {
        &mut self.base
    }

    fn focus_event(&mut self) {
        self.base.len = self.len();
    }

    fn get_internal_event(&self, key: &KeyCode) -> UIEvent {
        self.base.get_event(key)
    }
}
