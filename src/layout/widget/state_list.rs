use super::{widget_state::RCToDo, widget_trait::State, Widget};
use crate::utils::get_block;
use crate::{todo::ToDoData, CONFIG};
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    style::{Color, Style},
    widgets::{List, ListState},
    Frame,
};

pub struct StateList {
    state: ListState,
    style: Style,
    data_type: ToDoData,
    data: RCToDo,
    focus: bool,
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
        }
    }

    pub fn len(&self) -> usize {
        self.data.lock().unwrap().len(self.data_type)
    }

    pub fn act(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }
}

impl State for StateList {
    fn handle_key(&mut self, event: &KeyEvent) {
        if self.len() == 0 {
            return
        }
        match event.code {
            KeyCode::Char('j') => {
                let act = self.act() + 1;
                if self.len() > act {
                    self.state.select(Some(act));
                }
            }
            KeyCode::Char('k') => {
                let act = self.act();
                if 0 < act {
                    self.state.select(Some(act - 1));
                }
            }
            KeyCode::Char('U') => {
                log::info!("Swap task up");
                let act = self.act();
                if act > 0 {
                    self.data
                        .lock()
                        .unwrap()
                        .swap_tasks(self.data_type, act, act - 1);
                    self.state.select(Some(act - 1));
                }
            }
            KeyCode::Char('D') => {
                log::info!("Swap task down");
                let act = self.act() + 1;
                let len = self.len();
                if act < len {
                    self.data
                        .lock()
                        .unwrap()
                        .swap_tasks(self.data_type, act, act - 1);
                    self.state.select(Some(act));
                }
            }
            KeyCode::Char('x') => {
                let act = self.act();
                log::info!("Remove task with index {act}.");
                self.data.lock().unwrap().remove_task(self.data_type, act);
                let len = self.len();
                if len <= act && len > 0 {
                    self.state.select(Some(len - 1));
                }
            }
            KeyCode::Char('d') => {
                let act = self.act();
                log::info!("Move task with index {act}.");
                self.data.lock().unwrap().move_task(self.data_type, act);
                let len = self.len();
                if len <= act && len > 0 {
                    self.state.select(Some(len - 1));
                }
            }
            KeyCode::Char('g') => self.state.select(Some(0)),
            KeyCode::Char('G') => self.state.select(Some(self.len() - 1)),
            KeyCode::Enter => {
                self.data
                    .lock()
                    .unwrap()
                    .set_active(self.data_type, self.act());
            }
            _ => {}
        }
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>, _: bool, widget: &Widget) {
        let data = self.data.lock().unwrap();
        let filtered = data.get_filtered(self.data_type);
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
