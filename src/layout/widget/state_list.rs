use super::{widget_base::WidgetBase, widget_list::WidgetList, widget_trait::State};
use crate::{
    todo::{task_list::TaskSort, ToDo, ToDoData},
    ui::{HandleEvent, UIEvent},
};
use crossterm::event::KeyCode;
use tui::{backend::Backend, style::Style, widgets::List, Frame};

pub struct StateList {
    base: WidgetList,
    style: Style,
    pub data_type: ToDoData,
    sort_type: TaskSort,
}

impl StateList {
    pub fn new(
        base: WidgetList,
        data_type: ToDoData,
        style: Style,
        list_shift: usize,
        sort_type: TaskSort,
    ) -> Self {
        let mut s = Self {
            base,
            style,
            data_type,
            sort_type,
        };
        s.base.set_shift(list_shift); // TODO to constructor
        s
    }

    pub fn len(&self) -> usize {
        self.base.data().len(self.data_type)
    }

    fn swap_tasks(&mut self, first: usize, second: usize) {
        log::trace!("Swap tasks with indexes: {}, {}", first, second);
        self.base.data().swap_tasks(self.data_type, first, second);
    }

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
        let mut filtered = data.get_filtered(self.data_type);
        filtered.sort(self.sort_type);
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
