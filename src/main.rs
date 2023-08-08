#![allow(dead_code, unused_variables)]

mod config;
mod error;
mod layout;
mod todo;
mod utils;
mod file_worker;
mod ui;

use crate::config::Config;
use crate::todo::ToDo;
use crate::file_worker::FileWorker;
use crate::ui::UI;
use layout::{Layout, DEFAULT_LAYOUT};
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

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

    let mut ui = UI::new(Layout::from_str(DEFAULT_LAYOUT, todo).unwrap());
    ui.run()?;

    Ok(())
}
