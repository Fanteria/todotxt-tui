mod config;
mod error;
mod file_worker;
mod layout;
mod todo;
mod ui;

use crate::{config::Config, file_worker::FileWorker, todo::ToDo, ui::UI};
use file_worker::FileWorkerCommands;
use layout::Layout;
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

fn init_logging(config: &Config) -> Result<(), Box<dyn Error>> {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(&config.log_format)))
        .build(&config.log_file)?;
    let logging_config = LogConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(config.log_level))?;
    log4rs::init_config(logging_config)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::load_default();
    init_logging(&config)?;
    log::trace!("===== PROGRAM START =====");
    let todo = Arc::new(Mutex::new(ToDo::new(&config)));
    let file_worker = FileWorker::new(
        config.todo_path.clone(),
        config.archive_path.clone(),
        todo.clone(),
    );

    file_worker.load()?;
    let tx = file_worker.run(config.autosave_duration, config.file_watcher);

    log::trace!("Starting UI...");
    UI::new(
        Layout::from_str(&config.layout, todo.clone(), &config)?,
        todo.clone(),
        tx.clone(),
        &config,
    )
    .run()?;

    tx.send(FileWorkerCommands::Exit).unwrap();
    Ok(())
}
