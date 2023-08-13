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

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config as LogConfig, Root};

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

    // TODO solve log file by better way
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} [{h({l})}] {M}: {m}{n}")))
        .build("log.log")?;

    let config = LogConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder()
                   .appender("logfile")
                   .build(LevelFilter::Trace))?;

    log4rs::init_config(config)?;

    let mut ui = UI::new(Layout::from_str(DEFAULT_LAYOUT, todo.clone()).unwrap(), todo);
    ui.run()?;

    Ok(())
}
