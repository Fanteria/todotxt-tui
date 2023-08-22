use super::{widget_state::RCToDo, widget_trait::State, Widget};
use crate::{todo::ToDoData, utils::get_block};
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
    last: usize,
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
            last: 24,
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
}

impl State for StateList {
    fn handle_key(&mut self, event: &KeyEvent) {
        if self.len() == 0 {
            return;
        }
        match event.code {
            KeyCode::Char('j') => {
                let act = self.act();
                let len = self.len();
                if self.last - self.first <= act + 1 + SHIFT {
                    if self.last + 1 < len {
                        self.first += 1;
                        self.last += 1;
                    } else {
                        if self.last - self.first > act + 1 {
                            self.state.select(Some(act + 1));
                        }
                    }
                } else {
                    self.state.select(Some(act + 1));
                }
            }
            KeyCode::Char('k') => {
                let act = self.act();
                if act <= SHIFT {
                    if self.first > 0 {
                        self.first -= 1;
                        self.last -= 1;
                    } else {
                        if act > 0 {
                            self.state.select(Some(act - 1));
                        }
                    }
                } else {
                    self.state.select(Some(act - 1));
                }
            }
            KeyCode::Char('U') => {
                log::info!("Swap task up");
                let index = self.index();
                if index > 0 {
                    self.data
                        .lock()
                        .unwrap()
                        .swap_tasks(self.data_type, index, index - 1);
                    self.state.select(Some(self.act() - 1));
                    // TODO what is this behavior???
                }
            }
            KeyCode::Char('D') => {
                log::info!("Swap task down");
                let index = self.index() + 1;
                let len = self.len();
                if index < len {
                    self.data
                        .lock()
                        .unwrap()
                        .swap_tasks(self.data_type, index, index - 1);
                    self.state.select(Some(self.act() + 1));
                }
            }
            KeyCode::Char('x') => {
                let index = self.index();
                log::info!("Remove task with index {index}.");
                self.data.lock().unwrap().remove_task(self.data_type, index);
                let len = self.len();
                if len <= index && len > 0 {
                    self.state.select(Some(len - 1));
                }
            }
            KeyCode::Char('d') => {
                let index = self.index();
                log::info!("Move task with index {index}.");
                self.data.lock().unwrap().move_task(self.data_type, index);
                let len = self.len();
                if len <= index && len > 0 {
                    self.state.select(Some(len - 1));
                }
            }
            KeyCode::Char('g') => {
                self.state.select(Some(0));
                self.last -= self.first;
                self.first = 0;
            }
            KeyCode::Char('G') => {
                let shown_items = self.last - self.first - 1;
                self.first = shown_items;
                self.last = self.len() - 1;
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
        let filtered = filtered.range(self.first, self.last);
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
