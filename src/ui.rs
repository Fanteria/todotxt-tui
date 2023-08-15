use crate::file_worker::FileWorkerCommands;
use crate::utils::some_or_return;
use crate::ToDo;
use crate::CONFIG;
use crate::{layout::Layout, utils::get_block};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
    ExecutableCommand,
};
use std::io;
use std::io::Result as ioResult;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use tui::layout::Constraint;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Direction, Layout as tuiLayout, Rect},
    widgets::Paragraph,
    Terminal,
};

#[derive(PartialEq, Eq)]
enum Mode {
    Input,
    Normal,
}

pub struct UI {
    input: String,
    input_chunk: Rect,
    layout: Layout,
    mode: Mode,
    data: Arc<Mutex<ToDo>>,
    tx: Sender<FileWorkerCommands>,
}

impl UI {
    pub fn new(layout: Layout, data: Arc<Mutex<ToDo>>, tx: Sender<FileWorkerCommands>) -> UI {
        UI {
            input: String::new(),
            input_chunk: Rect::default(),
            layout,
            mode: Mode::Normal,
            data,
            tx,
        }
    }

    fn update_chunks(&mut self, main_chunk: Rect) {
        let layout = tuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(main_chunk);
        self.input_chunk = layout[0];
        self.layout.update_chunks(layout[1]);
    }

    pub fn run(&mut self) -> ioResult<()> {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let mut backend = CrosstermBackend::new(stdout);
        backend.execute(SetTitle(CONFIG.window_title.clone()))?;

        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;
        self.update_chunks(terminal.size()?);

        self.main_loop(&mut terminal)?;

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn main_loop<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> ioResult<()> {
        loop {
            self.draw(terminal)?;

            match event::read()? {
                Event::Resize(width, height) => {
                    self.update_chunks(Rect::new(0, 0, width, height));
                }
                Event::Key(event) => match self.mode {
                    Mode::Input => match event.code {
                        KeyCode::Enter => {
                            self.data.lock().unwrap().new_task(&self.input).unwrap(); // TODO fix
                            self.input.clear();
                            self.mode = Mode::Normal;
                        }
                        KeyCode::Char(c) => {
                            self.input.push(c);
                        }
                        KeyCode::Backspace => {
                            self.input.pop();
                        }
                        KeyCode::Esc => {
                            self.mode = Mode::Normal;
                            self.layout.focus();
                        }
                        KeyCode::Tab => {
                            self.autocomplete();
                        }
                        _ => {}
                    },
                    Mode::Normal => match event.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('I') => {
                            self.mode = Mode::Input;
                            self.layout.unfocus();
                        }
                        KeyCode::Char('L') => self.layout.right(),
                        KeyCode::Char('H') => self.layout.left(),
                        KeyCode::Char('K') => self.layout.up(),
                        KeyCode::Char('J') => self.layout.down(),
                        KeyCode::Char('S') => {
                            if let Err(e) = self.tx.send(FileWorkerCommands::Save) {
                                log::error!("Error while send signal to save todo list: {}", e);
                                // TODO show something on screen
                            }
                        }
                        KeyCode::Char('U') => {
                            if let Err(e) = self.tx.send(FileWorkerCommands::Load) {
                                log::error!("Error while send signal to load todo list: {}", e);
                                // TODO show something on screen
                            }
                        }
                        _ => self.layout.handle_key(&event),
                    },
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) -> ioResult<()> {
        terminal.draw(|f| {
            f.render_widget(
                Paragraph::new(self.input.clone())
                    .block(get_block("Input", self.mode == Mode::Input)),
                self.input_chunk,
            );
            self.layout.render(f);
        })?;
        Ok(())
    }

    fn autocomplete(&mut self) {
        let last_space_index = self.input.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let base = some_or_return!(self.input.get(last_space_index..));
        let category = some_or_return!(base.get(0..1));
        let pattern = some_or_return!(base.get(1..));

        let data = self.data.lock().unwrap();
        let list = match category {
            "+" => data.get_projects(),
            "@" => data.get_contexts(),
            "#" => data.get_hashtags(),
            _ => return,
        };

        if list.is_empty() {
            return;
        }

        let list = list.start_with(pattern);

        let same_start_index = |fst: &str, sec: &str| -> usize {
            for (i, (fst_char, sec_char)) in fst.chars().zip(sec.chars()).enumerate() {
                if fst_char != sec_char {
                    return i;
                }
            }
            std::cmp::min(fst.len(), sec.len())
        };
        if list.is_empty() {
            return;
        }

        let mut new_act = list[0].as_str();

        if list.len() != 1 {
            list.iter()
                .skip(1)
                .for_each(|item| new_act = &new_act[..same_start_index(new_act, item)]);
            self.input += &new_act[pattern.len()..];
        } else {
            self.input += &new_act[pattern.len()..];
            self.input += " ";
        }
    }
}
