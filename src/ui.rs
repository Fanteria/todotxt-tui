mod ui_event;

pub use ui_event::*;

use crate::{
    file_worker::FileWorkerCommands, layout::Layout, layout::Render, todo::ToDoCategory, ToDo,
    CONFIG,
};
use crossterm::{
    self,
    event::{self, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
    ExecutableCommand,
};
use std::{
    io::{self, Result as ioResult},
    sync::mpsc::Sender,
    sync::{Arc, Mutex},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout as tuiLayout, Rect},
    style::Style,
    widgets::Paragraph,
    widgets::{Block, BorderType, Borders},
    Terminal,
};
use tui_input::{backend::crossterm::EventHandler, Input};

/// Enum representing the different modes of the UI.
#[derive(PartialEq, Eq)]
enum Mode {
    Input,
    Edit,
    Normal,
}

/// The struct representing the UI for the application.
pub struct UI {
    input_chunk: Rect,
    tinput: Input,
    layout: Layout,
    mode: Mode,
    data: Arc<Mutex<ToDo>>,
    tx: Sender<FileWorkerCommands>,
    event_handler: EventHandlerUI,
    quit: bool,
}

impl UI {
    /// Creates a new instance of the UI.
    ///
    /// # Arguments
    ///
    /// * `layout` - The initial layout configuration for the UI.
    /// * `data` - Shared data representing the to-do list.
    /// * `tx` - Sender for communicating with the file worker.
    ///
    /// # Returns
    ///
    /// A new `UI` instance.
    pub fn new(layout: Layout, data: Arc<Mutex<ToDo>>, tx: Sender<FileWorkerCommands>) -> UI {
        UI {
            input_chunk: Rect::default(),
            tinput: Input::default(),
            layout,
            mode: Mode::Normal,
            data,
            tx,
            event_handler: CONFIG.window_keybind.clone(),
            quit: false,
        }
    }

    /// Updates the input chunk of the UI based on the main chunk's dimensions.
    ///
    /// This method recalculates the position and size of the input chunk based on the dimensions
    /// of the main chunk, ensuring proper rendering of the input field.
    ///
    /// # Arguments
    ///
    /// * `main_chunk` - The main chunk's dimensions, typically representing the entire terminal window.
    fn update_chunk(&mut self, main_chunk: Rect) {
        let layout = tuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(main_chunk);
        self.input_chunk = layout[0];
        self.layout.update_chunk(layout[1]);
    }

    /// Runs the user interface, handling setup and cleanup of terminal interactions.
    ///
    /// This method enables raw mode, sets up the terminal, and enters the main event loop.
    ///
    /// # Returns
    ///
    /// An `ioResult` indicating the success of running the user interface.
    pub fn run(&mut self) -> ioResult<()> {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let mut backend = CrosstermBackend::new(stdout);
        backend.execute(SetTitle(CONFIG.window_title.clone()))?;

        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;
        self.update_chunk(terminal.size()?);

        self.draw(&mut terminal)?;
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

    /// Handles the main event loop of the UI.
    ///
    /// # Arguments
    ///
    /// * `terminal` - The TUI Terminal.
    ///
    /// # Returns
    ///
    /// An `ioResult` indicating the success of the main loop.
    fn main_loop<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> ioResult<()> {
        let mut version = self.data.lock().unwrap().get_version();
        let mut new_version;
        loop {
            if event::poll(CONFIG.list_refresh_rate)? {
                if self.handle_event()? {
                    break;
                }
                version = self.data.lock().unwrap().get_version();
                self.draw(terminal)?;
            } else {
                new_version = self.data.lock().unwrap().get_version();
                if new_version != version {
                    version = self.data.lock().unwrap().get_version();
                    self.draw(terminal)?;
                }
            }
        }
        Ok(())
    }

    /// Draws the UI on the terminal.
    ///
    /// # Arguments
    ///
    /// * `terminal` - The TUI Terminal.
    ///
    /// # Returns
    ///
    /// An `ioResult` indicating the success of drawing.
    fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) -> ioResult<()> {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .title("Input")
            .border_type(BorderType::Rounded);
        if self.mode == Mode::Input {
            block = block.border_style(Style::default().fg(CONFIG.active_color));
        }
        terminal.draw(|f| {
            f.render_widget(
                // Paragraph::new(self.input.clone()).block(block),
                Paragraph::new(self.tinput.value()).block(block),
                self.input_chunk,
            );
            self.layout.render(f);
            if self.mode == Mode::Input {
                let width = self.input_chunk.width.max(3) - 3;
                let scroll = self.tinput.visual_scroll(width as usize);
                f.set_cursor(
                    self.input_chunk.x
                        + (self.tinput.visual_cursor().max(scroll) - scroll) as u16
                        + 1,
                    self.input_chunk.y + 1,
                );
            }
        })?;
        Ok(())
    }

    /// Handles autocompletion based on user input.
    fn autocomplete(&self, input: &str) -> String {
        macro_rules! some_or_return {
            ($message:expr) => {
                match $message {
                    Some(s) => s,
                    None => return input.to_string(),
                }
            };
        }

        let last_space_index = input.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let base = some_or_return!(input.get(last_space_index..));
        // let last_space_index = self.input.rfind(' ').map(|i| i + 1).unwrap_or(0);
        // let base = some_or_return!(self.input.get(last_space_index..));
        let category = some_or_return!(base.get(0..1));
        let pattern = some_or_return!(base.get(1..));

        let data = self.data.lock().unwrap();
        let list = match category {
            "+" => data.get_categories(ToDoCategory::Projects),
            "@" => data.get_categories(ToDoCategory::Contexts),
            "#" => data.get_categories(ToDoCategory::Hashtags),
            _ => return input.to_string(),
        };

        if list.is_empty() {
            return input.to_string();
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
            return input.to_string();
        }

        let mut new_act = list[0].as_str();
        if list.len() != 1 {
            list.iter()
                .skip(1)
                .for_each(|item| new_act = &new_act[..same_start_index(new_act, item)]);
            input.to_string() + &new_act[pattern.len()..]
        } else {
            input.to_string() + &new_act[pattern.len()..] + " "
        }
    }

    /// Handles various user events.
    ///
    /// # Returns
    ///
    /// An `ioResult` indicating whether the application should exit.
    fn handle_event(&mut self) -> ioResult<bool> {
        let e = read()?;
        match e {
            Event::Resize(width, height) => {
                self.update_chunk(Rect::new(0, 0, width, height));
            }
            Event::Key(event) => match self.mode {
                Mode::Input => match event.code {
                    KeyCode::Enter => {
                        self.data
                            .lock()
                            .unwrap()
                            .new_task(&self.tinput.value())
                            .unwrap(); // TODO fix
                        self.tinput.reset();
                        self.mode = Mode::Normal;
                        self.layout.focus();
                    }
                    KeyCode::Esc => {
                        self.mode = Mode::Normal;
                        self.layout.focus();
                    }
                    KeyCode::Tab => {
                        self.tinput = self.autocomplete(&self.tinput.value()).into();
                    }
                    _ => {
                        self.tinput.handle_event(&e);
                    }
                },
                Mode::Edit => match event.code {
                    KeyCode::Enter => {
                        self.data
                            .lock()
                            .unwrap()
                            .update_active(&self.tinput.value())
                            .unwrap();
                        self.tinput.reset();
                        self.mode = Mode::Normal;
                        self.layout.focus();
                    }
                    KeyCode::Esc => {
                        self.tinput.reset();
                        self.mode = Mode::Normal;
                        self.layout.focus();
                    }
                    KeyCode::Tab => {
                        self.tinput = self.autocomplete(&self.tinput.value()).into();
                    }
                    _ => {
                        self.tinput.handle_event(&e);
                    }
                },
                Mode::Normal => {
                    let _ = self.handle_key(&event.code) || self.layout.handle_key(&event);
                }
            },
            _ => {}
        }
        Ok(self.quit)
    }
}

impl HandleEvent for UI {
    fn get_event(&self, key: &KeyCode) -> UIEvent {
        self.event_handler.get_event(key)
    }

    fn handle_event(&mut self, event: UIEvent) -> bool {
        use UIEvent::*;
        match event {
            Quit => self.quit = true,
            InsertMode => {
                self.mode = Mode::Input;
                self.layout.unfocus();
            }
            MoveRight => self.layout.right(),
            MoveLeft => self.layout.left(),
            MoveUp => self.layout.up(),
            MoveDown => self.layout.down(),
            Save => {
                if let Err(e) = self.tx.send(FileWorkerCommands::ForceSave) {
                    log::error!("Error while send signal to save todo list: {}", e);
                    // TODO show something on screen
                }
            }
            Load => {
                if let Err(e) = self.tx.send(FileWorkerCommands::Load) {
                    log::error!("Error while send signal to load todo list: {}", e);
                    // TODO show something on screen
                }
            }
            EditMode => {
                if let Some(active) = self.data.lock().unwrap().get_active() {
                    self.tinput = active.to_string().into();
                    self.mode = Mode::Edit;
                    self.layout.unfocus();
                }
            }
            _ => {
                return false;
            }
        }
        true
    }
}
