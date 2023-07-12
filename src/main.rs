#![allow(dead_code, unused_variables)]

mod config;
mod error;
mod layout;
mod todo;
mod utils;
mod file_worker;

use crate::config::Config;
use crate::todo::ToDo;
use crate::file_worker::FileWorker;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
    ExecutableCommand,
};
use layout::{Layout, DEFAULT_LAYOUT};
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::error::Error;
use std::io;
use std::rc::Rc;
use tui::{backend::CrosstermBackend, layout::Rect, Terminal};

#[macro_use]
extern crate enum_dispatch;

lazy_static! {
    static ref CONFIG: Config = Config::load_default();
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut todo = ToDo::new(false);
    let file_worker = FileWorker::new(CONFIG.todo_path.clone(), CONFIG.archive_path.clone());
    file_worker.load(&mut todo)?;
    let todo = Rc::new(RefCell::new(todo));
    draw_ui(todo)?;

    Ok(())
}

fn draw_ui(data: Rc<RefCell<ToDo>>) -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let mut backend = CrosstermBackend::new(io::stdout());
    backend.execute(SetTitle(CONFIG.window_title.clone()))?;

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut layout = Layout::from_str(DEFAULT_LAYOUT, data).unwrap();
    layout.update_chunks(terminal.size()?);

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
