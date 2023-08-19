#![allow(dead_code, unused_variables)]

mod config;
mod error;
mod file_worker;
mod layout;
mod todo;
mod ui;
mod utils;

use crate::{config::Config, file_worker::FileWorker, todo::ToDo, ui::UI};
use layout::{Layout, DEFAULT_LAYOUT};
use lazy_static::lazy_static;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config as LogConfig, Root},
    encode::pattern::PatternEncoder,
};
use std::{
    error::Error,
    sync::{Arc, Mutex}, time::Duration,
};

#[macro_use]
extern crate enum_dispatch;

lazy_static! {
    static ref CONFIG: Config = Config::load_default();
}

fn main() -> Result<(), Box<dyn Error>> {
    // TODO solve log file by better way
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} [{h({l})}] {M}: {m}{n}")))
        .build("log.log")?;
    let config = LogConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Trace),
        )?;
    log4rs::init_config(config)?;

    let todo = Arc::new(Mutex::new(ToDo::new(false)));
    let file_worker = FileWorker::new(
        CONFIG.todo_path.clone(),
        CONFIG.archive_path.clone(),
        todo.clone(),
    );
    file_worker.load()?;

    // let tx = file_worker.run(CONFIG.autosave_duration);
    let tx = file_worker.run(Duration::from_secs(5));

    let mut ui = UI::new(
        Layout::from_str(DEFAULT_LAYOUT, todo.clone()).unwrap(),
        todo.clone(),
        tx.clone(),
    );

    ui.run()?;

    Ok(())
}
