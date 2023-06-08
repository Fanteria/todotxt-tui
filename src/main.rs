mod config;
mod error;
mod layout;
mod todo;
mod utils;

use crate::config::Config;
use crate::todo::ToDo;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
    ExecutableCommand,
};
use layout::Layout;
use lazy_static::lazy_static;
use std::error::Error;
use std::fs::File;
use std::io;
use std::rc::Rc;
use tui::{backend::CrosstermBackend, layout::Rect, Terminal};

#[macro_use]
extern crate enum_dispatch;

lazy_static! {
    static ref CONFIG: Config = Config::load_default();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let todo = Rc::new(ToDo::load(File::open(CONFIG.todo_path.clone())?, false)?);

    draw_ui(todo).await?;

    Ok(())
}

async fn draw_ui(data: Rc<ToDo>) -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let mut backend = CrosstermBackend::new(io::stdout());
    backend.execute(SetTitle(CONFIG.window_title.clone()))?;

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut layout = Layout::new(terminal.size()?, CONFIG.init_widget, data);

    // main loop
    loop {
        terminal.draw(|f| {
            layout.render(f);
        })?;
        if layout.cursor_visible() {
            terminal.show_cursor()?;
        }
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
                _ => {
                    layout.handle_key(&event)
                    // if let Some(widget) = layout.active_widget() {
                    //     widget.handle_key(&event);
                    //     println!("THIS IS NOT PRINTED");
                    // }
                }
            },
            _ => {}
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}
