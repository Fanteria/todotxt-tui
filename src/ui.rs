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
}

impl UI {
    pub fn new(layout: Layout) -> UI {
        UI {
            input: String::new(),
            input_chunk: Rect::default(),
            layout,
            mode: Mode::Normal,
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
                        }
                        _ => {}
                    },
                    Mode::Normal => match event.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('I') => self.mode = Mode::Input,
                        KeyCode::Char('L') => self.layout.right(),
                        KeyCode::Char('H') => self.layout.left(),
                        KeyCode::Char('K') => self.layout.up(),
                        KeyCode::Char('J') => self.layout.down(),
                        _ => {
                            self.layout.handle_key(&event)
                            // if let Some(widget) = layout.active_widget() {
                            //     widget.handle_key(&event);
                            //     println!("THIS IS NOT PRINTED");
                            // }
                        }
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
}
