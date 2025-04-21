use super::{widget_base::WidgetBase, widget_list::WidgetList, widget_trait::State};
use crate::{
    config::{ActiveColorConfig, TextStyle},
    todo::{search::Searchable, FilterState, ToDo, ToDoCategory},
    ui::UIEvent,
};
use crossterm::event::KeyEvent;
use tui::{widgets::List, Frame};

/// Represents the state for a widget that displays categories.
#[derive(Debug)]
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

    /// Number of items in the category associated with this widget.
    pub fn len(&self, todo: &ToDo) -> usize {
        todo.get_categories(self.category).len()
    }

    /// Toggles the filter state for a specific category.
    fn toggle_filter(&mut self, filter_state: FilterState, todo: &mut ToDo) {
        let name = todo
            .get_categories(self.category)
            .get_name(self.base.act())
            .to_string();
        todo.toggle_filter(self.category, &name, filter_state);
    }
}

impl State for StateCategories {
    fn handle_event_state(&mut self, event: UIEvent, todo: &mut ToDo) -> bool {
        if self.base.handle_event(event, self.len(todo)) {
            return true;
        }
        match event {
            UIEvent::Select => self.toggle_filter(FilterState::Select, todo),
            UIEvent::Remove => self.toggle_filter(FilterState::Remove, todo),
            UIEvent::NextSearch => {
                if let Some(to_search) = &self.base.to_search {
                    if let Some(next) = todo
                        .get_categories(self.category)
                        .next_search_index(to_search, self.base.index() + 1)
                    {
                        log::debug!("Search next: {} times down", next);
                        for _ in 0..next + 1 {
                            self.base.down(self.len(todo))
                        }
                    }
                }
            }
            UIEvent::PrevSearch => {
                if let Some(to_search) = &self.base.to_search {
                    if let Some(prev) = todo
                        .get_categories(self.category)
                        .prev_search_index(to_search, self.base.index())
                    {
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

    fn render(&self, f: &mut Frame, todo: &ToDo) {
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

    fn focus_event(&mut self, todo: &ToDo) -> bool {
        let len = self.len(todo);
        if self.base.act() >= len && len > 0 {
            self.base.last(self.len(todo));
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

    fn get_internal_event(&self, event: &KeyEvent) -> UIEvent {
        self.base.get_event(event)
    }

    fn handle_click(&mut self, column: usize, row: usize, todo: &ToDo) {
        self.base.click(column, row, self.len(todo));
    }

    fn widget_type(&self) -> super::widget_type::WidgetType {
        self.category.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, layout::widget::widget_type::WidgetType, todo::ToDo};
    use std::str::FromStr;

    #[test]
    fn handle_event_state() {
        let config = Config::default();
        let mut todo = ToDo::default();
        todo.add_task(todo_txt::Task::from_str("Task +project1").unwrap());
        todo.add_task(todo_txt::Task::from_str("Task +project1").unwrap());
        todo.add_task(todo_txt::Task::from_str("Task +project2").unwrap());
        todo.add_task(todo_txt::Task::from_str("Task +project3").unwrap());

        let mut c = StateCategories::new(
            WidgetList::new(&WidgetType::Project, &config),
            ToDoCategory::Projects,
            &config.active_color_config,
        );

        c.base.set_size(20);
        c.search_event(String::from("proj"));
        assert!(c.handle_event_state(UIEvent::NextSearch, &mut todo));
        assert_eq!(c.base.index(), 1);
        assert!(c.handle_event_state(UIEvent::NextSearch, &mut todo));
        assert_eq!(c.base.act(), 2);
        assert!(c.handle_event_state(UIEvent::NextSearch, &mut todo));
        assert_eq!(c.base.act(), 2);
        assert!(c.handle_event_state(UIEvent::NextSearch, &mut todo));
        assert_eq!(c.base.act(), 2);

        assert!(c.handle_event_state(UIEvent::PrevSearch, &mut todo));
        assert_eq!(c.base.act(), 1);
        assert!(c.handle_event_state(UIEvent::PrevSearch, &mut todo));
        assert_eq!(c.base.act(), 0);
        assert!(c.handle_event_state(UIEvent::PrevSearch, &mut todo));
        assert_eq!(c.base.act(), 0);

        c.clear_search();
        assert!(c.handle_event_state(UIEvent::NextSearch, &mut todo));
        assert_eq!(c.base.act(), 0);

        assert!(c.handle_event_state(UIEvent::Select, &mut todo));
        {
            assert_eq!(
                todo.get_state().get_category(c.category)["project1"],
                FilterState::Select
            );
        }

        assert!(c.handle_event_state(UIEvent::Remove, &mut todo));
        {
            assert_eq!(
                todo.get_state().get_category(c.category)["project1"],
                FilterState::Remove
            );
        }

        // Do nothing on unknown event
        assert!(!c.handle_event_state(UIEvent::Quit, &mut todo));
    }
}
