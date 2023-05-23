mod error;
mod layout;
mod config;

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
use crate::layout::widget::WidgetType;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    draw_ui().await?;

    Ok(())
}

async fn draw_ui() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let mut backend = CrosstermBackend::new(io::stdout());
    backend.execute(SetTitle("Title"))?; // TODO set window title

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut layout = Layout::new(terminal.size()?, WidgetType::List);
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
                KeyCode::Char('l') => layout.right(),
                KeyCode::Char('h') => layout.left(),
                KeyCode::Char('k') => layout.up(),
                KeyCode::Char('j') => layout.down(),
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
