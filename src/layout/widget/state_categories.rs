use super::{widget_base::WidgetBase, widget_list::WidgetList, widget_trait::State};
use crate::{
    config::{ActiveColorConfig, TextStyle},
    todo::{FilterState, ToDoCategory},
    ui::{HandleEvent, UIEvent},
};
use crossterm::event::KeyCode;
use tui::{backend::Backend, widgets::List, Frame};

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
}

impl State for StateCategories {
    fn handle_event_state(&mut self, event: UIEvent) -> bool {
        if self.base.handle_event(event) {
            return true;
        }
        let filter_state = match event {
            UIEvent::Select => FilterState::Select,
            UIEvent::Remove => FilterState::Remove,
            _ => return false,
        };
        let name = {
            let todo = self.base.data();
            todo.get_categories(self.category)
                .get_name(self.base.act())
                .clone()
        };
        self.base
            .data()
            .toggle_filter(self.category, &name, filter_state);
        self.base.len = self.len();
        true
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>) {
        let todo = self.base.data();
        let data = todo.get_categories(self.category);
        let (first, last) = self.base.range();
        let list = List::new(data.slice(first, last)).block(self.get_block());
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
        self.base.len = self.len();
        true
    }

    fn update_chunk_event(&mut self) {
        self.base.set_size(self.base.chunk.height - 2); // Two chars are borders.
    }

    fn get_internal_event(&self, key: &KeyCode) -> UIEvent {
        self.base.get_event(key)
    }

    fn handle_click(&mut self, column: usize, row: usize) {
        self.base.click(column, row);
    }
}
