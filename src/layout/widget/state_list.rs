use super::{widget_base::WidgetBase, widget_list::WidgetList, widget_trait::State};
use crate::{
    config::Config,
    todo::{ToDo, ToDoData},
    ui::{HandleEvent, UIEvent},
};
use crossterm::event::KeyCode;
use tui::{backend::Backend, style::Style, widgets::List, Frame};

/// Represents the state for a list widget that displays tasks.
pub struct StateList {
    base: WidgetList,
    style: Style,
    pub data_type: ToDoData,
}

impl StateList {
    /// Creates a new `StateList` instance.
    ///
    /// # Parameters
    ///
    /// - `base`: The base properties shared among different widget types.
    /// - `data_type`: The type of task data to display (e.g., Pending or Done tasks).
    /// - `style`: The style used to render the list widget.
    ///
    /// # Returns
    ///
    /// A new `StateList` instance.
    pub fn new(base: WidgetList, data_type: ToDoData, config: &Config) -> Self {
        Self {
            base,
            style: config
                .get_list_active_color()
                .combine(&match data_type {
                    ToDoData::Done => config.get_done_active_color(),
                    ToDoData::Pending => config.get_pending_active_color(),
                })
                .get_style(),
            data_type,
        }
    }

    /// Gets the number of tasks in the list.
    ///
    /// # Returns
    ///
    /// The number of tasks in the list.
    pub fn len(&self) -> usize {
        self.base.data().len(self.data_type)
    }

    /// Swaps tasks in the list at the selected and previous indices.
    ///
    /// # Parameters
    ///
    /// - `first`: The index of the first task to swap.
    /// - `second`: The index of the second task to swap.
    fn swap_tasks(&mut self, first: usize, second: usize) {
        log::trace!("Swap tasks with indexes: {}, {}", first, second);
        self.base.data().swap_tasks(self.data_type, first, second);
    }

    /// Moves the currently selected task using the specified function.
    ///
    /// # Parameters
    ///
    /// - `move_fn`: The function to move the task (e.g., remove or move).
    fn move_task(&mut self, r#move: fn(&mut ToDo, ToDoData, usize)) {
        let index = self.base.index();
        log::info!("Remove task with index {index}.");
        r#move(&mut self.base.data(), self.data_type, index);
        let len = self.len();
        if len <= index && len > 0 {
            self.base.up();
        }
        self.base.len = len;
    }
}

impl State for StateList {
    fn handle_event_state(&mut self, event: UIEvent) -> bool {
        if self.base.handle_event(event) {
            return true;
        }
        match event {
            UIEvent::SwapUpItem => {
                if let Some((first, second)) = self.base.prev() {
                    self.swap_tasks(first, second)
                }
            }
            UIEvent::SwapDownItem => {
                if let Some((first, second)) = self.base.next() {
                    self.swap_tasks(first, second)
                }
            }
            UIEvent::RemoveItem => self.move_task(ToDo::remove_task),
            UIEvent::MoveItem => self.move_task(ToDo::move_task),
            UIEvent::Select => {
                log::trace!("Set item on index {} active.", self.base.index());
                self.base
                    .data()
                    .set_active(self.data_type, self.base.index());
            }
            _ => return false,
        }
        true
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>) {
        let data = self.base.data();
        let filtered = data.get_filtered_and_sorted(self.data_type);
        let (first, last) = self.base.range();
        let filtered = filtered.slice(first, last);
        let list = List::new(filtered).block(self.get_block());
        if !self.base.focus {
            f.render_widget(list, self.base.chunk)
        } else {
            let list = list.highlight_style(self.style);
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
        let len = self.len();
        self.base.len = len;
        if self.base.act() >= len && len > 0 {
            self.base.last();
        }
    }

    fn update_chunk_event(&mut self) {
        self.base.set_size(self.base.chunk.height - 2); // Two chars are borders.
    }

    fn get_internal_event(&self, key: &KeyCode) -> UIEvent {
        self.base.get_event(key)
    }
}
