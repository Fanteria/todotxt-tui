use super::{widget_state::RCToDo, widget_trait::State, Widget};
use crate::{
    todo::{ToDo, ToDoData},
    utils::get_block,
};
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    style::Style,
    widgets::{List, ListState},
    Frame,
};

const SHIFT: usize = 4;

pub struct StateList {
    state: ListState,
    style: Style,
    data_type: ToDoData,
    data: RCToDo,
    focus: bool,
    first: usize,
    size: usize,
}

impl StateList {
    pub fn new(data_type: ToDoData, data: RCToDo, style: Style) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            style,
            data_type,
            data,
            focus: false,
            first: 0,
            size: 24,
        }
    }

    pub fn len(&self) -> usize {
        self.data.lock().unwrap().len(self.data_type)
    }

    pub fn act(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }

    pub fn index(&self) -> usize {
        self.act() + self.first
    }

    fn move_list_down(&mut self) {
        let act = self.act();
        let len = self.len();
        log::trace!("List go down: act: {}, len: {}", act, len);
        if len <= self.size {
            if len > act + 1 {
                self.state.select(Some(act + 1));
            }
        } else if self.size <= act + 1 + SHIFT {
            if self.first + self.size + 1 < len {
                self.first += 1;
            } else {
                if self.size > act + 1 {
                    self.state.select(Some(act + 1));
                }
            }
        } else {
            self.state.select(Some(act + 1));
        }
    }

    fn move_list_up(&mut self) {
        let act = self.act();
        log::trace!("List go up: act: {}", act);
        if act <= SHIFT {
            if self.first > 0 {
                self.first -= 1;
            } else {
                if act > 0 {
                    self.state.select(Some(act - 1));
                }
            }
        } else {
            self.state.select(Some(act - 1));
        }
    }

    fn swap_tasks(&mut self, r#move_list: fn(&mut Self)) {
        let index_first = self.index();
        r#move_list(self);
        let index_second = self.index();
        log::trace!("Swap tasks with indexes: {}, {}", index_first, index_second);
        self.data
            .lock()
            .unwrap()
            .swap_tasks(self.data_type, index_first, index_second);
    }

    fn move_task(&mut self, r#move: fn(&mut ToDo, ToDoData, usize)) {
        let index = self.index();
        log::info!("Remove task with index {index}.");
        r#move(&mut self.data.lock().unwrap(), self.data_type, index);
        let len = self.len();
        if len <= index && len > 0 {
            self.state.select(Some(len - 1));
        }
    }
}

impl State for StateList {
    fn handle_key(&mut self, event: &KeyEvent) {
        if self.len() == 0 {
            return;
        }
        match event.code {
            KeyCode::Char('j') => self.move_list_down(),
            KeyCode::Char('k') => self.move_list_up(),
            KeyCode::Char('U') => {
                log::info!("Swap task up");
                let act = self.act();
                if act > 0 {
                    self.swap_tasks(Self::move_list_up)
                }
            }
            KeyCode::Char('D') => {
                log::info!("Swap task down");
                let act = self.act();
                if act + 1 < self.len() {
                    self.swap_tasks(Self::move_list_down)
                }
            }
            KeyCode::Char('x') => self.move_task(ToDo::remove_task),
            KeyCode::Char('d') => self.move_task(ToDo::move_task),
            KeyCode::Char('g') => {
                self.state.select(Some(0));
                self.first = 0;
            }
            KeyCode::Char('G') => {
                let shown_items = self.size - 1;
                self.first = shown_items;
                self.state.select(Some(shown_items));
            }
            KeyCode::Enter => {
                self.data
                    .lock()
                    .unwrap()
                    .set_active(self.data_type, self.index());
            }
            _ => {}
        }
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>, _: bool, widget: &Widget) {
        let data = self.data.lock().unwrap();
        let filtered = data.get_filtered(self.data_type);
        let filtered = filtered.slice(self.first, self.first + self.size);
        let list = List::new(filtered).block(get_block(&widget.title, self.focus));
        if !self.focus {
            f.render_widget(list, widget.chunk)
        } else {
            let list = list.highlight_style(self.style);
            f.render_stateful_widget(list, widget.chunk, &mut self.state.clone());
        }
    }

    fn focus(&mut self) {
        self.focus = true;
        let len = self.len();
        if self.act() >= len && len > 0 {
            self.state.select(Some(len - 1));
        }
    }

    fn unfocus(&mut self) {
        self.focus = false;
    }
}
