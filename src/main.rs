mod error;
mod layout;
mod config;
mod todo;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
    ExecutableCommand,
};
use layout::Layout;
use std::io;
use tui::{backend::CrosstermBackend, layout::Rect, Terminal};
use lazy_static::lazy_static;
use crate::config::Config;
use crate::todo::ToDo;
use std::fs::File;

lazy_static! {
    static ref CONFIG: Config = Config::load_default();
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let todo = ToDo::load(File::open(CONFIG.todo_path.clone())?, false);
    
    draw_ui().await?;

    Ok(())
}

async fn draw_ui() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let mut backend = CrosstermBackend::new(io::stdout());
    backend.execute(SetTitle(CONFIG.window_title.clone()))?;

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut layout = Layout::new(terminal.size()?, CONFIG.init_widget);
    terminal.draw(|f| {
        layout.render(f);
    })?;

    // main loop
    loop {
        match event::read()? {
            Event::Resize(width, height) => {
                layout.update_chunks(Rect::new(0, 0, width, height));
            }
            Event::Key(event) => match event.code {
                KeyCode::Char('q') => {
                    break;
                }
                KeyCode::Char('L') => layout.right(),
                KeyCode::Char('H') => layout.left(),
                KeyCode::Char('K') => layout.up(),
                KeyCode::Char('J') => layout.down(),
                _ => {}
            },
            _ => {}
        }

        terminal.draw(|f| {
            layout.render(f);
        })?;
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}
