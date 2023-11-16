mod config;
mod error;
mod file_worker;
mod layout;
mod todo;
mod ui;

use crate::{config::Config, file_worker::FileWorker, todo::ToDo, ui::UI};
use file_worker::FileWorkerCommands;
use layout::Layout;
use lazy_static::lazy_static;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config as LogConfig, Root},
    encode::pattern::PatternEncoder,
};
use std::{
    error::Error,
    sync::{Arc, Mutex},
};

#[macro_use]
extern crate enum_dispatch;

lazy_static! {
    static ref CONFIG: Config = Config::load_default();
}

fn init_logging() -> Result<(), Box<dyn Error>> {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(&CONFIG.log_format)))
        .build(&CONFIG.log_file)?;
    let config = LogConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(CONFIG.log_level))?;
    log4rs::init_config(config)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    init_logging()?;
    log::trace!("===== PROGRAM START =====");
    let mut todo = ToDo::new(false);
    todo.pending_sort = CONFIG.pending_sort;
    todo.done_sort = CONFIG.done_sort;
    let todo = Arc::new(Mutex::new(todo));
    let file_worker = FileWorker::new(
        CONFIG.todo_path.clone(),
        CONFIG.archive_path.clone(),
        todo.clone(),
    );

    file_worker.load()?;
    let tx = file_worker.run(CONFIG.autosave_duration, CONFIG.file_watcher);

    log::trace!("Starting UI...");
    UI::new(
        Layout::from_str(&CONFIG.layout, todo.clone(), &CONFIG)?,
        todo.clone(),
        tx.clone(),
    )
    .run()?;

    tx.send(FileWorkerCommands::Exit).unwrap();
    Ok(())
}
