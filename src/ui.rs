mod handle_event_trait;
mod popup;
mod ui_event;
mod ui_state;

use popup::Popup;

pub use handle_event_trait::HandleEvent;
pub use ui_event::*;
pub use ui_state::UIState;

use crate::{
    config::{Config, PasteBehavior, UiConfig, WidgetBorderType},
    file_worker::{FileWorker, FileWorkerCommands},
    layout::{Layout, Render},
    todo::{autocomplete, ToDo},
    Result,
};
use crossterm::{
    event::{
        self, read, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste,
        EnableMouseCapture, Event, KeyCode, MouseEvent,
    },
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
    ///
    /// # Arguments
    ///
    /// * `config`: A reference to a `Config` struct containing all necessary settings for building the UI.
    ///
    /// # Returns
    ///
    /// * On success, returns an `Ok(UI)` containing the newly built UI.
    /// * On failure, returns an `Err(Result<(), ErrorKind>)`.
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

        let mut layout = Layout::from_str(&config.ui_config.layout, &todo, config)?;

        let todo = Arc::new(Mutex::new(todo));
        let file_worker = FileWorker::new(config.file_worker_config.clone(), todo.clone())?;

        file_worker.load()?;
        let tx = file_worker.run()?;

        layout.select_widget(init_widget, &todo.lock().unwrap());

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
        fn restore_tui(enable_mouse: bool, paste_behavior: PasteBehavior) -> io::Result<()> {
            disable_raw_mode()?;
            execute!(io::stdout(), LeaveAlternateScreen,)?;
            if enable_mouse {
                execute!(io::stdout(), DisableMouseCapture,)?;
            }
            if paste_behavior != PasteBehavior::AsKeys {
                execute!(io::stdout(), DisableBracketedPaste,)?;
            }
            Ok(())
        }

        fn run_ui(this: &mut UI) -> Result<()> {
            // setup terminal
            enable_raw_mode()?;
            execute!(io::stdout(), EnterAlternateScreen,)?;
            if this.config.enable_mouse {
                execute!(io::stdout(), EnableMouseCapture,)?;
            }
            if this.config.paste_behavior != PasteBehavior::AsKeys {
                execute!(io::stdout(), EnableBracketedPaste,)?;
            }

            let mut backend = CrosstermBackend::new(io::stdout());
            backend.execute(SetTitle(this.config.window_title.clone()))?;

            let mut terminal = Terminal::new(backend)?;
            terminal.hide_cursor()?;
            let size = terminal.size()?;
            this.update_chunk(Rect::new(0, 0, size.width, size.height));

            this.draw(&mut terminal)?;
            this.main_loop(&mut terminal)?;

            // restore terminal
            restore_tui(this.config.enable_mouse, this.config.paste_behavior)?;
            terminal.show_cursor()?;

            Ok(())
        }

        // Setup panic hook.
        let orig_hook = take_hook();
        let enable_mouse = self.config.enable_mouse;
        let paste_behavior = self.config.paste_behavior;
        set_hook(Box::new(move |panic_info| {
            let _ = restore_tui(enable_mouse, paste_behavior);
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
    ///
    /// # Arguments
    ///
    /// * `terminal` - The TUI Terminal.
    ///
    /// # Returns
    ///
    /// An `Result` indicating the success of the main loop.
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
    ///
    /// # Arguments
    ///
    /// * `terminal` - The TUI Terminal.
    ///
    /// # Returns
    ///
    /// An `Result` indicating the success of drawing.
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
            self.layout.render(f, &self.data.lock().unwrap());

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
    ///
    /// # Returns
    ///
    /// An `Result` indicating whether the application should exit.
    fn process_event(&mut self) -> Result<bool> {
        self.handle_event_window(read()?);
        Ok(self.quit)
    }

    /// Handles window events, such as resizing and mouse clicks, to manage the
    /// user interface state.
    ///
    /// # Arguments
    ///
    /// * `e`: The event that triggers the function, which can be a resize event
    ///   or a mouse click event.
    ///
    /// # Details
    ///
    /// This function processes different types of events:
    /// - **Resize Event**: Adjusts the UI chunk based on the new window dimensions.
    /// - **Mouse Click Event**: Triggers a click action in the layout manager
    ///   at the specified column and row.
    /// - **Keyboard Events**: Depending on the current mode (`Mode::Input`, `Mode::Edit`,
    ///   or `Mode::Normal`), handles input for task creation, editing, and general navigation
    ///   using specific keys.
    fn handle_event_window(&mut self, e: Event) {
        match (&e, &self.mode) {
            (Event::Resize(width, height), _) => {
                log::debug!("Resize event: width {width}, height {height}");
                self.update_chunk(Rect::new(0, 0, *width, *height));
            }
            (
                Event::Mouse(MouseEvent {
                    kind: event::MouseEventKind::Up(event::MouseButton::Left),
                    column,
                    row,
                    modifiers: _,
                }),
                _,
            ) => {
                log::debug!("Mouse event: column {column}, row {row}");
                self.layout.click(*column, *row, &self.data.lock().unwrap());
            }
            (Event::Paste(s), Mode::Normal) => {
                if self.config.paste_behavior == PasteBehavior::Insert {
                    log::debug!("Pasted: {s}");
                    if let Err(e) = self.data.lock().unwrap().new_task(s) {
                        log::error!("Error while pasting new task: {e}");
                        self.popup.add_message(format!("Failed paste task: {e}"));
                    }
                }
            }
            (Event::Paste(s), _) => self.tinput = Input::new(self.tinput.value().to_owned() + s),
            (Event::Key(event), Mode::Input) => match event.code {
                KeyCode::Enter => {
                    if let Err(e) = self.data.lock().unwrap().new_task(self.tinput.value()) {
                        log::error!("Error while adding new task: {e}");
                        self.popup.add_message(format!("Failed add task: {e}"));
                    }
                    self.tinput.reset();
                    self.mode = Mode::Normal;
                    self.layout.focus(&self.data.lock().unwrap());
                }
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                    self.layout.focus(&self.data.lock().unwrap());
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
            (Event::Key(event), Mode::Edit) => match event.code {
                KeyCode::Enter => {
                    if let Err(e) = self.data.lock().unwrap().update_active(self.tinput.value()) {
                        log::error!("Error while updating existing task: {e}");
                        self.popup.add_message(format!("Failed update task: {e}"));
                    }
                    self.tinput.reset();
                    self.mode = Mode::Normal;
                    self.layout.focus(&self.data.lock().unwrap());
                }
                KeyCode::Esc => {
                    self.tinput.reset();
                    self.mode = Mode::Normal;
                    self.layout.focus(&self.data.lock().unwrap());
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
            (Event::Key(event), Mode::Search) => match event.code {
                KeyCode::Enter => {
                    self.mode = Mode::Normal;
                    self.layout.focus(&self.data.lock().unwrap());
                    self.tinput.reset();
                }
                KeyCode::Esc => {
                    self.tinput.reset();
                    self.mode = Mode::Normal;
                    self.layout.clean_search();
                    self.layout.focus(&self.data.lock().unwrap());
                }
                _ => {
                    self.tinput.handle_event(&e);
                    self.layout.search(self.tinput.to_string())
                }
            },
            (Event::Key(event), Mode::Normal) => {
                log::debug!("Handle event: {:?}", event);
                if !self.handle(event) {
                    self.layout
                        .handle_key(event, &mut self.data.lock().unwrap());
                }
            }
            _ => {}
        }
    }

    fn handle(&mut self, event: &event::KeyEvent) -> bool {
        use UIEvent::*;
        match self.config.window_keybinds.get_event(event) {
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
                self.layout.right(&self.data.lock().unwrap());
            }
            MoveLeft => {
                self.layout.left(&self.data.lock().unwrap());
            }
            MoveUp => {
                self.layout.up(&self.data.lock().unwrap());
            }
            MoveDown => {
                self.layout.down(&self.data.lock().unwrap());
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

// impl HandleEvent for UI {
//     fn get_event(&self, event: &KeyEvent) -> UIEvent {
//         log::debug!("get_event {:#?}", self.config.window_keybinds);
//         self.config.window_keybinds.get_event(event)
//     }
//
//     fn handle_event(&mut self, event: UIEvent, todo: &mut ToDo) -> bool {
//         use UIEvent::*;
//         match event {
//             Quit => {
//                 if let Some(path) = &self.config.save_state_path {
//                     if let Err(e) =
//                         UIState::new(&self.layout, &self.data.lock().unwrap()).save(path)
//                     {
//                         log::error!("Error while saving UI state: {}", e);
//                     }
//                 }
//                 self.quit = true;
//             }
//             InsertMode => {
//                 self.mode = Mode::Input;
//                 self.layout.unfocus();
//             }
//             MoveRight => {
//                 self.layout.right(todo);
//             }
//             MoveLeft => {
//                 self.layout.left(todo);
//             }
//             MoveUp => {
//                 self.layout.up(todo);
//             }
//             MoveDown => {
//                 self.layout.down(todo);
//             }
//             Save => {
//                 if let Err(e) = self.tx.send(FileWorkerCommands::ForceSave) {
//                     log::error!("Error while send signal to save todo list: {e}");
//                     self.popup
//                         .add_message(format!("Cannot save todo list: {e}"));
//                 }
//             }
//             Load => {
//                 if let Err(e) = self.tx.send(FileWorkerCommands::Load) {
//                     log::error!("Error while send signal to load todo list: {e}");
//                     self.popup
//                         .add_message(format!("Cannot load todo list: {e}"));
//                 }
//             }
//             EditMode => {
//                 if let Some(active) = self.data.lock().unwrap().get_active() {
//                     self.tinput = active.to_string().into();
//                     self.mode = Mode::Edit;
//                     self.layout.unfocus();
//                     // self.in
//                 }
//             }
//             SearchMode => {
//                 self.tinput.reset();
//                 self.mode = Mode::Search;
//                 self.layout.unfocus();
//             }
//             _ => {
//                 return false;
//             }
//         }
//         true
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Conf, layout::widget::widget_type::WidgetType};
    use crossterm::event::KeyEvent;
    use std::{env, str::FromStr};
    use test_log::test;

    macro_rules! handle_event {
        ($ui:expr, $code:expr) => {
            let key_shortcut = KeyShortcut::from_str($code)?;
            let event = Event::Key(KeyEvent::new(key_shortcut.key, key_shortcut.modifiers));
            $ui.handle_event_window(event);
        };
    }

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

        handle_event!(ui, "S+j");
        assert_eq!(ui.layout.get_active_widget(), WidgetType::List);

        handle_event!(ui, "S+l");
        assert_eq!(ui.layout.get_active_widget(), WidgetType::Done);

        handle_event!(ui, "S+k");
        assert_eq!(ui.layout.get_active_widget(), WidgetType::Done);

        handle_event!(ui, "S+l");
        assert_eq!(ui.layout.get_active_widget(), WidgetType::Done);

        handle_event!(ui, "S+j");
        assert_eq!(ui.layout.get_active_widget(), WidgetType::Context);

        handle_event!(ui, "S+h");
        assert_eq!(ui.layout.get_active_widget(), WidgetType::List);

        Ok(())
    }

    #[test]
    fn test_behaviour() -> Result<()> {
        let mut ui = default_ui()?;
        ui.update_chunk(Rect::new(0, 0, 20, 20));

        let event = Event::Resize(50, 50);
        ui.handle_event_window(event);

        handle_event!(ui, "j");
        assert_eq!(ui.layout.get_active_widget(), WidgetType::List);

        handle_event!(ui, "Enter");
        assert!(ui.data.lock().unwrap().get_active().is_some());

        handle_event!(ui, "S+i");
        assert_eq!(ui.mode, Mode::Input);

        handle_event!(ui, "Esc");
        assert_eq!(ui.mode, Mode::Normal);

        handle_event!(ui, "S+e");
        assert_eq!(ui.mode, Mode::Edit);

        handle_event!(ui, "Esc");
        assert_eq!(ui.mode, Mode::Normal);

        handle_event!(ui, "/");
        assert_eq!(ui.mode, Mode::Search);

        handle_event!(ui, "Esc");
        assert_eq!(ui.mode, Mode::Normal);

        handle_event!(ui, "/");
        assert_eq!(ui.mode, Mode::Search);

        handle_event!(ui, "a");
        assert_eq!(ui.mode, Mode::Search);

        handle_event!(ui, "Enter");
        assert_eq!(ui.mode, Mode::Normal);

        handle_event!(ui, "u");

        handle_event!(ui, "S+i");
        assert_eq!(ui.mode, Mode::Input);

        handle_event!(ui, "a");
        assert_eq!(ui.tinput.to_string(), "a");
        assert_eq!(ui.mode, Mode::Input);
        handle_event!(ui, "b");
        assert_eq!(ui.tinput.to_string(), "ab");
        assert_eq!(ui.mode, Mode::Input);
        handle_event!(ui, "c");
        assert_eq!(ui.tinput.to_string(), "abc");
        assert_eq!(ui.mode, Mode::Input);
        handle_event!(ui, "Tab");
        assert_eq!(ui.tinput.to_string(), "abc");
        assert_eq!(ui.mode, Mode::Input);

        handle_event!(ui, "Enter");
        assert_eq!(ui.tinput.to_string(), "");
        assert_eq!(ui.mode, Mode::Normal);

        handle_event!(ui, "S+e");
        assert_eq!(ui.mode, Mode::Edit);

        handle_event!(ui, " ");
        assert_eq!(ui.tinput.to_string(), "Second task ");
        assert_eq!(ui.mode, Mode::Edit);
        handle_event!(ui, "plus");
        assert_eq!(ui.tinput.to_string(), "Second task +");
        assert_eq!(ui.mode, Mode::Edit);
        handle_event!(ui, "a");
        assert_eq!(ui.tinput.to_string(), "Second task +a");
        assert_eq!(ui.mode, Mode::Edit);
        handle_event!(ui, "Tab");
        assert_eq!(ui.tinput.to_string(), "Second task +abcdef ");
        assert_eq!(ui.mode, Mode::Edit);
        handle_event!(ui, "Backspace");
        handle_event!(ui, "Backspace");
        handle_event!(ui, "Backspace");
        handle_event!(ui, "Backspace");
        handle_event!(ui, "Backspace");
        handle_event!(ui, "Backspace");
        handle_event!(ui, "Backspace");
        handle_event!(ui, "Backspace");
        handle_event!(ui, "Backspace");

        handle_event!(ui, "Enter");
        assert_eq!(ui.tinput.to_string(), "");
        assert_eq!(ui.mode, Mode::Normal);

        // Remove items added in this test
        handle_event!(ui, "S+g");
        handle_event!(ui, "x");
        handle_event!(ui, "x");
        handle_event!(ui, "x");

        // Quit TUI.
        assert!(!ui.quit);
        handle_event!(ui, "q");
        assert!(ui.quit);
        ui.quit = false;

        Ok(())
    }

    #[test]
    fn search_contexts() -> Result<()> {
        let mut ui = default_ui()?;
        ui.update_chunk(Rect::new(0, 0, 20, 20));

        handle_event!(ui, "S+l");
        assert_eq!(ui.layout.get_active_widget(), WidgetType::Done);

        handle_event!(ui, "S+j");
        assert_eq!(ui.layout.get_active_widget(), WidgetType::Context);

        handle_event!(ui, "/");
        assert_eq!(ui.mode, Mode::Search);

        handle_event!(ui, "a");
        assert_eq!(ui.mode, Mode::Search);

        handle_event!(ui, "b");
        assert_eq!(ui.mode, Mode::Search);

        handle_event!(ui, "Enter");
        assert_eq!(ui.mode, Mode::Normal);

        // clean search
        handle_event!(ui, "h");
        assert_eq!(ui.mode, Mode::Normal);

        Ok(())
    }
}
