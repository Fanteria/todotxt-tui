use super::{widget_state::RCToDo, widget_trait::State, Widget};
use crate::todo::{TaskList, ToDo, ToDoData};
use crate::utils::get_block;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    widgets::{List, ListState},
    Frame,
};

pub struct StateList {
    state: ListState,
    fn_data: fn(&ToDo) -> TaskList,
    fn_move: fn(&mut ToDo, usize),
    data_type: ToDoData,
    data: RCToDo,
    focus: bool,
}

impl StateList {
    pub fn new(
        fn_data: fn(&ToDo) -> TaskList,
        fn_move: fn(&mut ToDo, usize),
        data_type: ToDoData,
        data: RCToDo,
    ) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            state,
            fn_data,
            fn_move,
            data_type,
            data,
            focus: false,
        }
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
                if (self.fn_data)(&self.data.lock().unwrap()).len() > act {
                    self.state.select(Some(act));
                }
            }
            KeyCode::Char('k') => {
                let act = self.state.selected().unwrap_or(0);
                if 0 < act {
                    self.state.select(Some(act - 1));
                }
            }
            KeyCode::Char('U') => {
                log::info!("Swap task up");
                if let Some(act) = self.state.selected() {
                    if act > 0 {
                        self.data
                            .lock()
                            .unwrap()
                            .swap_tasks(self.data_type, act, act - 1);
                        self.state.select(Some(act - 1));
                    }
                };
            }
            KeyCode::Char('D') => {
                log::info!("Swap task down");
                if let Some(act) = self.state.selected() {
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
                };
            }
            KeyCode::Char('x') => {
                if let Some(i) = self.state.selected() {
                    log::info!("Remove task with index {i}.");
                    self.data.lock().unwrap().remove_task(self.data_type, i);
                    let len = match &self.data_type {
                        ToDoData::Pending => self.data.lock().unwrap().pending.len(),
                        ToDoData::Done => self.data.lock().unwrap().done.len(),
                    };
                    if len <= i {
                        self.state.select(Some(len - 1));
                    }
                }
            }
            KeyCode::Char('d') => {
                if let Some(i) = self.state.selected() {
                    log::info!("Move task with index {i}.");
                    (self.fn_move)(&mut self.data.lock().unwrap(), i);
                    let len = match &self.data_type {
                        ToDoData::Pending => self.data.lock().unwrap().pending.len(),
                        ToDoData::Done => self.data.lock().unwrap().done.len(),
                    };
                    if len <= i {
                        self.state.select(Some(len - 1));
                    }
                }
            }
            KeyCode::Char('g') => {
                let len = self.data.lock().unwrap().len(self.data_type);
                if len > 0 {
                    self.state.select(Some(0))
                }
            }
            KeyCode::Char('G') => {
                let len = self.data.lock().unwrap().len(self.data_type);
                if len > 0 {
                    self.state.select(Some(len - 1))
                }
            }
            KeyCode::Enter => {
                if let Some(i) = self.state.selected() {
                    self.data.lock().unwrap().set_active(self.data_type, i);
                }
            }
            _ => {}
        }
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, widget: &Widget) {
        let todo = self.data.lock().unwrap();
        let data = (self.fn_data)(&todo);
        let list = List::new(data).block(get_block(&widget.title, self.focus));
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
