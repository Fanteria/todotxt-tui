mod ui_event;
mod ui_state;

pub use ui_event::*;
pub use ui_state::*;

use crate::{
    config::{Config, UiConfig},
    file_worker::{FileWorker, FileWorkerCommands},
    layout::{Layout, Render},
    todo::{autocomplete, ToDo},
    IOError, Result,
};
use crossterm::{
    self,
    event::{self, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEvent},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
    ExecutableCommand,
};
use std::{
    io,
    sync::mpsc::Sender,
    sync::{Arc, Mutex},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout as tuiLayout, Rect},
    style::{Color, Style},
    widgets::Paragraph,
    widgets::{Block, BorderType, Borders},
    Terminal,
};
use tui_input::{backend::crossterm::EventHandler, Input};

/// Enum representing the different modes of the UI.
#[derive(Debug, PartialEq, Eq)]
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
    quit: bool,
    active_color: Color,
    config: UiConfig,
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
    pub fn new(
        layout: Layout,
        data: Arc<Mutex<ToDo>>,
        tx: Sender<FileWorkerCommands>,
        config: &Config,
    ) -> UI {
        UI {
            input_chunk: Rect::default(),
            tinput: Input::default(),
            layout,
            mode: Mode::Normal,
            data,
            tx,
            quit: false,
            active_color: *config.styles.active_color,
            config: config.ui_config.clone(),
        }
    }

    pub fn build(config: &Config) -> Result<UI> {
        let mut todo = ToDo::new(config.todo_config.clone(), config.styles.clone());

        if let Some(path) = &config.ui_config.save_state_path {
            let state = UIState::load(path)?;
            let (_active, todo_state) = (state.active, state.todo_state);
            todo.update_state(todo_state);
        }

        let todo = Arc::new(Mutex::new(todo));
        let file_worker = FileWorker::new(config, todo.clone());

        file_worker.load()?;
        let tx = file_worker.run();

        let layout = Layout::from_str(&config.ui_config.layout, todo.clone(), config)?;

        Ok(UI::new(layout, todo, tx.clone(), config))
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
    /// An `Result` indicating the success of running the user interface.
    pub fn run(&mut self) -> Result<()> {
        fn run_ui(this: &mut UI) -> Result<()> {
            // setup terminal
            enable_raw_mode().map_err(IOError)?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(IOError)?;

            let mut backend = CrosstermBackend::new(stdout);
            backend
                .execute(SetTitle(this.config.window_title.clone()))
                .map_err(IOError)?;

            let mut terminal = Terminal::new(backend).map_err(IOError)?;
            terminal.hide_cursor().map_err(IOError)?;
            this.update_chunk(terminal.size().map_err(IOError)?);

            this.draw(&mut terminal)?;
            this.main_loop(&mut terminal)?;

            // restore terminal
            disable_raw_mode().map_err(IOError)?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )
            .map_err(IOError)?;
            terminal.show_cursor().map_err(IOError)?;

            Ok(())
        }

        if let Err(e) = run_ui(self) {
            self.tx.send(FileWorkerCommands::Exit).unwrap();
            Err(e)
        } else {
            Ok(())
        }
    }

    /// Handles the main event loop of the UI.
    ///
    /// # Arguments
    ///
    /// * `terminal` - The TUI Terminal.
    ///
    /// # Returns
    ///
    /// An `Result` indicating the success of the main loop.
    fn main_loop<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut version = self.data.lock().unwrap().get_version();
        let mut new_version;
        loop {
            if event::poll(self.config.list_refresh_rate).map_err(IOError)? {
                if self.process_event()? {
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
    /// An `Result` indicating the success of drawing.
    fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .title("Input")
            .border_type(BorderType::Rounded);
        if self.mode == Mode::Input || self.mode == Mode::Edit {
            block = block.border_style(Style::default().fg(self.active_color));
        }
        terminal
            .draw(|f| {
                f.render_widget(
                    Paragraph::new(self.tinput.value()).block(block),
                    self.input_chunk,
                );
                self.layout.render(f);

                if self.mode == Mode::Input || self.mode == Mode::Edit {
                    let width = self.input_chunk.width.max(3) - 3;
                    let scroll = self.tinput.visual_scroll(width as usize);
                    f.set_cursor(
                        self.input_chunk.x
                            + (self.tinput.visual_cursor().max(scroll) - scroll) as u16
                            + 1,
                        self.input_chunk.y + 1,
                    );
                }
            })
            .map_err(IOError)?;
        Ok(())
    }

    /// Handles various user events.
    ///
    /// # Returns
    ///
    /// An `Result` indicating whether the application should exit.
    fn process_event(&mut self) -> Result<bool> {
        self.handle_event_window(read().map_err(IOError)?);
        Ok(self.quit)
    }

    fn handle_event_window(&mut self, e: Event) {
        match e {
            Event::Resize(width, height) => {
                log::debug!("Resize event: width {width}, height {height}");
                self.update_chunk(Rect::new(0, 0, width, height));
            }
            Event::Mouse(MouseEvent {
                kind: event::MouseEventKind::Up(event::MouseButton::Left),
                column,
                row,
                modifiers: _,
            }) => {
                log::debug!("Mouse event: column {column}, row {row}");
                self.layout.click(column, row);
            }
            Event::Key(event) => match self.mode {
                Mode::Input => match event.code {
                    KeyCode::Enter => {
                        if let Err(e) = self.data.lock().unwrap().new_task(self.tinput.value()) {
                            log::error!("Error while adding new task: {e}");
                            // TODO show something on screen
                        }
                        self.tinput.reset();
                        self.mode = Mode::Normal;
                        self.layout.focus();
                    }
                    KeyCode::Esc => {
                        self.mode = Mode::Normal;
                        self.layout.focus();
                    }
                    KeyCode::Tab => {
                        if let Some(input) =
                            autocomplete(&self.data.lock().unwrap(), self.tinput.value())
                        {
                            self.tinput = input.into();
                        }
                    }
                    _ => {
                        self.tinput.handle_event(&e);
                    }
                },
                Mode::Edit => match event.code {
                    KeyCode::Enter => {
                        if let Err(e) = self.data.lock().unwrap().update_active(self.tinput.value())
                        {
                            log::error!("Error while updating existing task: {e}");
                            // TODO show something on screen
                        }
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
                        if let Some(input) =
                            autocomplete(&self.data.lock().unwrap(), self.tinput.value())
                        {
                            self.tinput = input.into();
                        }
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
    }
}

impl HandleEvent for UI {
    fn get_event(&self, key: &KeyCode) -> UIEvent {
        self.config.window_keybinds.get_event(key)
    }

    fn handle_event(&mut self, event: UIEvent) -> bool {
        use UIEvent::*;
        match event {
            Quit => {
                if let Some(path) = &self.config.save_state_path {
                    if let Err(e) =
                        UIState::new(&self.layout, &self.data.lock().unwrap()).save(path)
                    {
                        log::error!("Error while saveing UI state: {}", e);
                    }
                }
                self.quit = true;
            }
            InsertMode => {
                self.mode = Mode::Input;
                self.layout.unfocus();
            }
            MoveRight => {
                self.layout.right();
            }
            MoveLeft => {
                self.layout.left();
            }
            MoveUp => {
                self.layout.up();
            }
            MoveDown => {
                self.layout.down();
            }
            Save => {
                if let Err(e) = self.tx.send(FileWorkerCommands::ForceSave) {
                    log::error!("Error while send signal to save todo list: {e}");
                    // TODO show something on screen
                }
            }
            Load => {
                if let Err(e) = self.tx.send(FileWorkerCommands::Load) {
                    log::error!("Error while send signal to load todo list: {e}");
                    // TODO show something on screen
                }
            }
            EditMode => {
                if let Some(active) = self.data.lock().unwrap().get_active() {
                    self.tinput = active.to_string().into();
                    self.mode = Mode::Edit;
                    self.layout.unfocus();
                    // self.in
                }
            }
            _ => {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Conf;
    use crossterm::event::{KeyEvent, KeyModifiers};
    use std::env;
    use std::error::Error;
    use test_log::test;

    fn default_ui() -> Result<UI> {
        let config = Config::from_reader(
            format!(
                r#"
            todo_path = "{}todo.txt"

            [[list_keybind.events]]
            event = "ListDown"
            key.Char = "j"

            [[list_keybind.events]]
            event = "Select"
            key = "Enter"

            [[list_keybind.events]]
            event = "InsertMode"
            key.Char = "I"

            [[list_keybind.events]]
            event = "EditMode"
            key.Char = "E"

            [[list_keybind.events]]
            event = "Quit"
            key.Char = "q"

            [[list_keybind.events]]
            event = "Save"
            key.Char = "S"

            [[list_keybind.events]]
            event = "Load"
            key.Char = "L"
            "#,
                env::var("TODO_TUI_TEST_DIR")?
            )
            .as_bytes(),
        )?;
        UI::build(&config)
    }

    #[test]
    fn test_behaviour() -> std::result::Result<(), Box<dyn Error>> {
        let mut ui = default_ui()?;
        ui.update_chunk(Rect::new(0, 0, 20, 20));

        let event = Event::Resize(50, 50);
        ui.handle_event_window(event);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        ui.handle_event_window(event);

        let event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        ui.handle_event_window(event);
        // assert!(ui.data.lock().unwrap().get_active().is_some());

        // let event = Event::Key(KeyEvent::new(KeyCode::Char('I'), KeyModifiers::NONE));
        // ui.handle_event_window(event);
        // assert_eq!(ui.mode, Mode::Input);
        //
        // let event = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        // ui.handle_event_window(event);
        // assert_eq!(ui.mode, Mode::Normal);
        //
        // let event = Event::Key(KeyEvent::new(KeyCode::Char('E'), KeyModifiers::NONE));
        // ui.handle_event_window(event);
        // assert_eq!(ui.mode, Mode::Edit);
        //
        // let event = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        // ui.handle_event_window(event);
        // assert_eq!(ui.mode, Mode::Normal);
        //
        // assert!(!ui.quit);
        // let event = Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        // ui.handle_event_window(event);
        // assert!(ui.quit);
        // ui.quit = false;
        //
        // let event = Event::Key(KeyEvent::new(KeyCode::Char('S'), KeyModifiers::NONE));
        // ui.handle_event_window(event);
        //
        // let event = Event::Key(KeyEvent::new(KeyCode::Char('L'), KeyModifiers::NONE));
        // ui.handle_event_window(event);

        Ok(())
    }
}
