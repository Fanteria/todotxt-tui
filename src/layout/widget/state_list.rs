use super::{widget_base::WidgetBase, widget_list::WidgetList, widget_trait::State};
use crate::{
    todo::{task_list::TaskSort, ToDo, ToDoData},
    ui::{HandleEvent, UIEvent},
};
use crossterm::event::KeyCode;
use tui::{backend::Backend, style::Style, widgets::List, Frame};

pub struct StateList {
    state: WidgetList,
    style: Style,
    pub data_type: ToDoData,
    sort_type: TaskSort,
    base: WidgetBase,
}

impl StateList {
    pub fn new(
        base: WidgetBase,
        data_type: ToDoData,
        style: Style,
        list_shift: usize,
        sort_type: TaskSort,
    ) -> Self {
        let mut state = WidgetList::default();
        state.set_shift(list_shift);
        Self {
            state,
            style,
            data_type,
            sort_type,
            base,
        }
    }

    pub fn len(&self) -> usize {
        self.data().len(self.data_type)
    }

    fn swap_tasks(&mut self, first: usize, second: usize) {
        log::trace!("Swap tasks with indexes: {}, {}", first, second);
        self.data().swap_tasks(self.data_type, first, second);
    }

    fn move_task(&mut self, r#move: fn(&mut ToDo, ToDoData, usize)) {
        let index = self.state.index();
        log::info!("Remove task with index {index}.");
        r#move(&mut self.data(), self.data_type, index);
        let len = self.len();
        if len <= index && len > 0 {
            self.state.up();
        }
    }
}

impl State for StateList {
    fn handle_event_state(&mut self, event: UIEvent) -> bool {
        if self.state.handle_event(event) {
            return true;
        }
        match event {
            UIEvent::SwapUpItem => {
                if let Some((first, second)) = self.state.prev() {
                    self.swap_tasks(first, second)
                }
            }
            UIEvent::SwapDownItem => {
                if let Some((first, second)) = self.state.next(self.len()) {
                    self.swap_tasks(first, second)
                }
            }
            UIEvent::RemoveItem => self.move_task(ToDo::remove_task),
            UIEvent::MoveItem => self.move_task(ToDo::move_task),
            UIEvent::Select => {
                self.data().set_active(self.data_type, self.state.index());
            }
            _ => return false,
        }
        true
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>) {
        let data = self.data();
        let mut filtered = data.get_filtered(self.data_type);
        filtered.sort(self.sort_type);
        let (first, last) = self.state.range();
        let filtered = filtered.slice(first, last);
        let list = List::new(filtered).block(self.get_block());
        if !self.base.focus {
            f.render_widget(list, self.base.chunk)
        } else {
            let list = list.highlight_style(self.style);
            f.render_stateful_widget(list, self.base.chunk, &mut self.state.state());
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
        if self.state.act() >= len && len > 0 {
            self.state.last(len);
        }
    }

    fn update_chunk_event(&mut self) {
        self.state.set_size(self.base.chunk.height);
    }

    fn get_internal_event(&self, key: &KeyCode) -> UIEvent {
        self.state.get_event(key)
    }
}
