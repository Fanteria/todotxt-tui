use super::{widget_state::RCToDo, widget_trait::State, Widget};
use crate::todo::ToDoData;
use crate::utils::get_block;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    widgets::{List, ListState},
    Frame,
};

pub struct StateList {
    state: ListState,
    data_type: ToDoData,
    data: RCToDo,
    focus: bool,
}

impl StateList {
    pub fn new(data_type: ToDoData, data: RCToDo) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            state,
            data_type,
            data,
            focus: false,
        }
    }

    pub fn len(&self) -> usize {
        self.data.lock().unwrap().get_data(self.data_type).len()
    }

    pub fn act(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }
}

impl State for StateList {
    fn handle_key(&mut self, event: &KeyEvent) {
        match event.code {
            KeyCode::Char('j') => {
                let act = match self.state.selected() {
                    Some(a) => a + 1,
                    None => 0,
                };
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
                let act = self.act();
                let act = act + 1;
                let len = match &self.data_type {
                    ToDoData::Pending => self.data.lock().unwrap().pending.len(),
                    ToDoData::Done => self.data.lock().unwrap().done.len(),
                };
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
                let len = match &self.data_type {
                    ToDoData::Pending => self.data.lock().unwrap().pending.len(),
                    ToDoData::Done => self.data.lock().unwrap().done.len(),
                };
                if len <= act {
                    self.state.select(Some(len - 1));
                }
            }
            KeyCode::Char('d') => {
                let act = self.act();
                log::info!("Move task with index {act}.");
                self.data.lock().unwrap().move_task(self.data_type, act);
                let len = self.len();
                if len <= act {
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

    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, widget: &Widget) {
        let data = self.data.lock().unwrap();
        let filtered = data.get_filtered(self.data_type);
        let list = List::new(filtered).block(get_block(&widget.title, self.focus));
        if !self.focus {
            f.render_widget(list, widget.chunk)
        } else {
            // .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            let list = list.highlight_symbol(">>");
            f.render_stateful_widget(list, widget.chunk, &mut self.state.clone());
        }
    }

    fn focus(&mut self) {
        self.focus = true;
    }

    fn unfocus(&mut self) {
        self.focus = false;
    }

    fn cursor_visible(&self) -> bool {
        false
    }
}
