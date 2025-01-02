mod handle_event_trait;
mod popup;
mod ui_event;
mod ui_state;

pub use handle_event_trait::HandleEvent;
use popup::Popup;
pub use ui_event::*;
pub use ui_state::UIState;

use crate::{
    config::{Config, UiConfig, WidgetBorderType},
    file_worker::{FileWorker, FileWorkerCommands},
    layout::{Layout, Render},
    todo::{autocomplete, ToDo},
    Result,
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
    panic::{set_hook, take_hook},
    sync::{mpsc::Sender, Arc, Mutex},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout as tuiLayout, Position, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use tui_input::{backend::crossterm::EventHandler, Input};

/// Enum representing the different modes of the UI.
#[derive(Debug, PartialEq, Eq)]
enum Mode {
    Input,
    Edit,
    Search,
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
    border_type: WidgetBorderType,
    popup: Popup,
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
            border_type: config.widget_base_config.border_type,
            popup: Popup::new(config.widget_base_config.border_type),
        }
    }

    /// Builds a new `UI` instance using the provided configuration.
    pub fn build(config: &Config) -> Result<UI> {
        let mut todo = ToDo::new(
            config.todo_config.clone(),
            config.hook_paths.clone(),
            config.styles.clone(),
        );

        let mut init_widget = config.ui_config.init_widget;
        if let Some(path) = &config.ui_config.save_state_path {
            match UIState::load(path) {
                Ok(UIState { active, todo_state }) => {
                    todo.update_state(todo_state);
                    init_widget = active;
                }
                Err(e) => log::error!("Cannot load state: {e}"),
            }
        }

        let todo = Arc::new(Mutex::new(todo));
        let file_worker = FileWorker::new(config.file_worker_config.clone(), todo.clone());

        file_worker.load()?;
        let tx = file_worker.run()?;

        let mut layout = Layout::from_str(&config.ui_config.layout, todo.clone(), config)?;

        layout.select_widget(init_widget);

        Ok(UI::new(layout, todo, tx.clone(), config))
    }

    /// Updates the input chunk of the UI based on the main chunk's dimensions.
    ///
    /// This method recalculates the position and size of the input chunk based on the dimensions
    /// of the main chunk, ensuring proper rendering of the input field.
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
    pub fn run(&mut self) -> Result<()> {
        fn restore_tui() -> io::Result<()> {
            disable_raw_mode()?;
            execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
            Ok(())
        }

        fn run_ui(this: &mut UI) -> Result<()> {
            // setup terminal
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

            let mut backend = CrosstermBackend::new(stdout);
            backend.execute(SetTitle(this.config.window_title.clone()))?;

            let mut terminal = Terminal::new(backend)?;
            terminal.hide_cursor()?;
            let size = terminal.size()?;
            this.update_chunk(Rect::new(0, 0, size.width, size.height));

            this.draw(&mut terminal)?;
            this.main_loop(&mut terminal)?;

            // restore terminal
            restore_tui()?;
            terminal.show_cursor()?;

            Ok(())
        }

        // Setup panic hook.
        let orig_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = restore_tui();
            orig_hook(panic_info);
        }));

        if let Err(e) = run_ui(self) {
            self.tx.send(FileWorkerCommands::Exit).unwrap();
            Err(e)
        } else {
            Ok(())
        }
    }

    /// Handles the main event loop of the UI.
    fn main_loop<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut versions = self.data.lock().unwrap().get_version().get_version_all();
        loop {
            if event::poll(self.config.list_refresh_rate)? {
                if self.process_event()? {
                    break;
                }
                versions = self.data.lock().unwrap().get_version().get_version_all();
                self.draw(terminal)?;
            } else if !self
                .data
                .lock()
                .unwrap()
                .get_version()
                .is_actual_all(versions)
            {
                versions = self.data.lock().unwrap().get_version().get_version_all();
                self.draw(terminal)?;
            }
        }
        Ok(())
    }

    /// Draws the UI on the terminal.
    fn draw<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .title("Input")
            .border_type(self.border_type.into());
        if self.mode == Mode::Input || self.mode == Mode::Edit || self.mode == Mode::Search {
            block = block.border_style(Style::default().fg(self.active_color));
        }
        terminal.draw(|f| {
            f.render_widget(
                Paragraph::new(self.tinput.value()).block(block),
                self.input_chunk,
            );
            self.layout.render(f);

            if self.mode == Mode::Input || self.mode == Mode::Edit {
                let width = self.input_chunk.width.max(3) - 3;
                let scroll = self.tinput.visual_scroll(width as usize);
                f.set_cursor_position(Position {
                    x: self.input_chunk.x
                        + (self.tinput.visual_cursor().max(scroll) - scroll) as u16
                        + 1,
                    y: self.input_chunk.y + 1,
                });
            }

            self.popup.render_popup(f);
        })?;
        Ok(())
    }

    /// Handles various user events.
    fn process_event(&mut self) -> Result<bool> {
        self.handle_event_window(read()?);
        Ok(self.quit)
    }

    /// Handles window events, such as resizing and mouse clicks, to manage the
    /// user interface state.
    ///
    /// This function processes different types of events:
    /// - **Resize Event**: Adjusts the UI chunk based on the new window dimensions.
    /// - **Mouse Click Event**: Triggers a click action in the layout manager
    ///   at the specified column and row.
    /// - **Keyboard Events**: Depending on the current mode (`Mode::Input`, `Mode::Edit`,
    ///   or `Mode::Normal`), handles input for task creation, editing, and general navigation
    ///   using specific keys.
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
                            self.popup.add_message(format!("Failed add task: {e}"));
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
                            self.popup.add_message(format!("Failed update task: {e}"));
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
                Mode::Search => match event.code {
                    KeyCode::Enter => {
                        self.mode = Mode::Normal;
                        self.layout.focus();
                        self.tinput.reset();
                    }
                    KeyCode::Esc => {
                        self.tinput.reset();
                        self.mode = Mode::Normal;
                        self.layout.clean_search();
                        self.layout.focus();
                    }
                    _ => {
                        self.tinput.handle_event(&e);
                        self.layout.search(self.tinput.to_string())
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
                        log::error!("Error while saving UI state: {}", e);
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
                    self.popup
                        .add_message(format!("Cannot save todo list: {e}"));
                }
            }
            Load => {
                if let Err(e) = self.tx.send(FileWorkerCommands::Load) {
                    log::error!("Error while send signal to load todo list: {e}");
                    self.popup
                        .add_message(format!("Cannot load todo list: {e}"));
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
            SearchMode => {
                self.tinput.reset();
                self.mode = Mode::Search;
                self.layout.unfocus();
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
    use crate::{config::Conf, layout::widget::widget_type::WidgetType};
    use crossterm::event::{KeyEvent, KeyModifiers};
    use std::env;
    use test_log::test;

    fn default_ui() -> Result<UI> {
        let config = Config::from_reader(
            format!(
                r#"
            todo_path = "{}test_behaviour_todo.txt"
            save_state_path = "/this/path/does/not/exists"
            save_policy = "ManualOnly"

            [list_keybind]
            E = "EditMode"
            Enter = "Select"
            I = "InsertMode"
            u = "Load"
            S = "Save"
            j = "ListDown"
            q = "Quit"
            "#,
                env::var("TODO_TUI_TEST_DIR")?
            )
            .as_bytes(),
        )?;
        UI::build(&config)
    }

    #[test]
    fn test_moves() -> Result<()> {
        let mut ui = default_ui()?;
        ui.update_chunk(Rect::new(0, 0, 20, 20));

        let event = Event::Key(KeyEvent::new(KeyCode::Char('J'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.layout.get_active_widget(), WidgetType::List);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('L'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.layout.get_active_widget(), WidgetType::Done);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('K'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.layout.get_active_widget(), WidgetType::Done);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('L'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.layout.get_active_widget(), WidgetType::Done);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('J'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.layout.get_active_widget(), WidgetType::Context);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('H'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.layout.get_active_widget(), WidgetType::List);

        Ok(())
    }

    #[test]
    fn test_behaviour() -> Result<()> {
        let mut ui = default_ui()?;
        ui.update_chunk(Rect::new(0, 0, 20, 20));

        let event = Event::Resize(50, 50);
        ui.handle_event_window(event);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.layout.get_active_widget(), WidgetType::List);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('S'), KeyModifiers::NONE));
        ui.handle_event_window(event);

        let event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert!(ui.data.lock().unwrap().get_active().is_some());

        let event = Event::Key(KeyEvent::new(KeyCode::Char('I'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Input);

        let event = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Normal);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('E'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Edit);

        let event = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Normal);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Search);

        let event = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Normal);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Search);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Search);

        let event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Normal);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE));
        ui.handle_event_window(event);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('I'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Input);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.tinput.to_string(), "a");
        assert_eq!(ui.mode, Mode::Input);
        let event = Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.tinput.to_string(), "ab");
        assert_eq!(ui.mode, Mode::Input);
        let event = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.tinput.to_string(), "abc");
        assert_eq!(ui.mode, Mode::Input);
        let event = Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.tinput.to_string(), "abc");
        assert_eq!(ui.mode, Mode::Input);

        let event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.tinput.to_string(), "");
        assert_eq!(ui.mode, Mode::Normal);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('E'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Edit);

        let event = Event::Key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.tinput.to_string(), "Second task ");
        assert_eq!(ui.mode, Mode::Edit);
        let event = Event::Key(KeyEvent::new(KeyCode::Char('+'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.tinput.to_string(), "Second task +");
        assert_eq!(ui.mode, Mode::Edit);
        let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.tinput.to_string(), "Second task +a");
        assert_eq!(ui.mode, Mode::Edit);
        let event = Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.tinput.to_string(), "Second task +abcdef ");
        assert_eq!(ui.mode, Mode::Edit);
        let event = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        ui.handle_event_window(event);
        let event = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        ui.handle_event_window(event);
        let event = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        ui.handle_event_window(event);
        let event = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        ui.handle_event_window(event);
        let event = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        ui.handle_event_window(event);
        let event = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        ui.handle_event_window(event);
        let event = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        ui.handle_event_window(event);
        let event = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        ui.handle_event_window(event);
        let event = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        ui.handle_event_window(event);

        let event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.tinput.to_string(), "");
        assert_eq!(ui.mode, Mode::Normal);

        // Remove items added in this test
        let event = Event::Key(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        let event = Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        let event = Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
        ui.handle_event_window(event);

        // Quit TUI.
        assert!(!ui.quit);
        let event = Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert!(ui.quit);
        ui.quit = false;

        Ok(())
    }

    #[test]
    fn search_contexts() -> Result<()> {
        let mut ui = default_ui()?;
        ui.update_chunk(Rect::new(0, 0, 20, 20));

        let event = Event::Key(KeyEvent::new(KeyCode::Char('L'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.layout.get_active_widget(), WidgetType::Done);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('J'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.layout.get_active_widget(), WidgetType::Context);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Search);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Search);

        let event = Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Search);

        let event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Normal);

        // clean search
        let event = Event::Key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
        ui.handle_event_window(event);
        assert_eq!(ui.mode, Mode::Normal);

        Ok(())
    }
}
