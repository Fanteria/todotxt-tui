use super::{widget_base::WidgetBase, widget_list::WidgetList, widget_trait::State};
use crate::{
    config::{ActiveColorConfig, TextStyle},
    todo::{search::Search, FilterState, ToDoCategory},
    ui::UIEvent,
};
use crossterm::event::KeyCode;
use tui::{widgets::List, Frame};

/// Represents the state for a widget that displays categories.
pub struct StateCategories {
    base: WidgetList,
    pub category: ToDoCategory,
    style: TextStyle,
}

impl StateCategories {
    /// Creates a new `StateCategories` instance.
    ///
    /// # Parameters
    ///
    /// - `base`: The base properties shared among different widget types.
    /// - `category`: The category of tasks to display.
    ///
    /// # Returns
    ///
    /// A new `StateCategories` instance.
    pub fn new(base: WidgetList, category: ToDoCategory, active_color: &ActiveColorConfig) -> Self {
        log::error!("{:?}", active_color.get_active_config_style(&category));
        Self {
            base,
            category,
            style: active_color.get_active_config_style(&category),
        }
    }

    /// Returns the number of items in the category associated with this widget.
    ///
    /// # Returns
    ///
    /// The number of items in the category.
    pub fn len(&self) -> usize {
        self.base.data().get_categories(self.category).len()
    }

    fn toggle_filter(&mut self, filter_state: FilterState) {
        let name = {
            let todo = self.base.data();
            todo.get_categories(self.category)
                .get_name(self.base.act())
                .clone()
        };
        self.base
            .data()
            .toggle_filter(self.category, &name, filter_state);
    }
}

impl State for StateCategories {
    fn handle_event_state(&mut self, event: UIEvent) -> bool {
        if self.base.handle_event(event, self.len()) {
            return true;
        }
        match event {
            UIEvent::Select => self.toggle_filter(FilterState::Select),
            UIEvent::Remove => self.toggle_filter(FilterState::Remove),
            UIEvent::NextSearch => {
                if let Some(to_search) = &self.base.to_search {
                    let next = {
                        let todo = self.base.data();
                        let data = todo.get_categories(self.category);
                        let next = Search::find(
                            data.vec.iter().skip(self.base.index() + 1).enumerate(),
                            to_search,
                            |c| c.1.name,
                        );
                        next.map(|next| next.0)
                    };
                    if let Some(next) = next {
                        log::debug!("Search next: {} times down", next);
                        for _ in 0..next + 1 {
                            self.base.down(self.len())
                        }
                    }
                }
            }
            UIEvent::PrevSearch => {
                if let Some(to_search) = &self.base.to_search {
                    let prev = {
                        let todo = self.base.data();
                        let data = todo.get_categories(self.category);
                        let prev = Search::find(
                            data.vec
                                .iter()
                                .rev()
                                .skip(data.vec.len() - self.base.index())
                                .enumerate(),
                            to_search,
                            |t| t.1.name,
                        );
                        prev.map(|prev| prev.0)
                    };
                    if let Some(prev) = prev {
                        log::debug!("Search prev: {} times up", prev);
                        for _ in 0..prev + 1 {
                            self.base.up()
                        }
                    }
                }
            }
            _ => return false,
        };
        true
    }

    fn render(&self, f: &mut Frame) {
        let todo = self.base.data();
        let data = todo.get_categories(self.category);
        let (first, last) = self.base.range();
        let list = List::from(data.get_view(first..last, self.base.to_search.as_deref()))
            .block(self.get_block());
        if !self.base.focus {
            f.render_widget(list, self.base.chunk)
        } else {
            f.render_stateful_widget(
                list.highlight_style(self.style.get_style()),
                self.base.chunk,
                &mut self.base.state(),
            );
        }
    }

    fn get_base(&self) -> &WidgetBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut WidgetBase {
        &mut self.base
    }

    fn focus_event(&mut self) -> bool {
        let len = self.len();
        if self.base.act() >= len && len > 0 {
            self.base.last(self.len());
        }
        true
    }

    fn search_event(&mut self, to_search: String) {
        self.base.set_search(to_search);
    }

    fn clear_search(&mut self) {
        self.base.clear_search();
    }

    fn update_chunk_event(&mut self) {
        self.base.set_size(self.base.chunk.height - 2); // Two chars are borders.
    }

    fn get_internal_event(&self, key: &KeyCode) -> UIEvent {
        self.base.get_event(key)
    }

    fn handle_click(&mut self, column: usize, row: usize) {
        self.base.click(column, row, self.len());
    }
}

#[cfg(test)]
mod tests {
    use std::{
        str::FromStr,
        sync::{Arc, Mutex},
    };

    use super::*;
    use crate::{config::Config, layout::widget::widget_type::WidgetType, todo::ToDo};

    #[test]
    fn handle_event_state() {
        let config = Config::default();
        let mut todo = ToDo::default();
        todo.add_task(todo_txt::Task::from_str("Task +project1").unwrap());
        todo.add_task(todo_txt::Task::from_str("Task +project1").unwrap());
        todo.add_task(todo_txt::Task::from_str("Task +project2").unwrap());
        todo.add_task(todo_txt::Task::from_str("Task +project3").unwrap());

        let mut c = StateCategories::new(
            WidgetList::new(&WidgetType::Project, Arc::new(Mutex::new(todo)), &config),
            ToDoCategory::Projects,
            &config.active_color_config,
        );

        c.base.set_size(20);
        c.search_event(String::from("proj"));
        assert!(c.handle_event_state(UIEvent::NextSearch));
        assert_eq!(c.base.index(), 1);
        assert!(c.handle_event_state(UIEvent::NextSearch));
        assert_eq!(c.base.act(), 2);
        assert!(c.handle_event_state(UIEvent::NextSearch));
        assert_eq!(c.base.act(), 2);
        assert!(c.handle_event_state(UIEvent::NextSearch));
        assert_eq!(c.base.act(), 2);

        assert!(c.handle_event_state(UIEvent::PrevSearch));
        assert_eq!(c.base.act(), 1);
        assert!(c.handle_event_state(UIEvent::PrevSearch));
        assert_eq!(c.base.act(), 0);
        assert!(c.handle_event_state(UIEvent::PrevSearch));
        assert_eq!(c.base.act(), 0);

        c.clear_search();
        assert!(c.handle_event_state(UIEvent::NextSearch));
        assert_eq!(c.base.act(), 0);

        assert!(c.handle_event_state(UIEvent::Select));
        {
            let todo = c.base.data();
            assert_eq!(
                todo.get_state().get_category(c.category)["project1"],
                FilterState::Select
            );
        }

        assert!(c.handle_event_state(UIEvent::Remove));
        {
            let todo = c.base.data();
            assert_eq!(
                todo.get_state().get_category(c.category)["project1"],
                FilterState::Remove
            );
        }

        // Do nothing on unknown event
        assert!(!c.handle_event_state(UIEvent::Quit));
    }
}
