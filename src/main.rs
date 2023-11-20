mod config;
mod error;
mod file_worker;
mod layout;
mod todo;
mod ui;

use crate::{
    config::{Config, Logger},
    file_worker::FileWorker,
    todo::ToDo,
    ui::UI,
};
use clap::Parser;
use file_worker::FileWorkerCommands;
use layout::Layout;
use std::{
    error::Error,
    sync::{Arc, Mutex},
};

#[macro_use]
extern crate enum_dispatch;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::parse();
    let load_config = config.load_config()?;
    let config = config.merge(load_config);
    if config.export()? {
        return Ok(())
    }

    Logger::new(&config).init()?;
    log::trace!("===== PROGRAM START =====");
    let todo = Arc::new(Mutex::new(ToDo::new(&config)));
    let file_worker = FileWorker::new(
        config.get_todo_path(),
        config.get_archive_path(),
        todo.clone(),
    );

    file_worker.load()?;
    let tx = file_worker.run(config.get_autosave_duration(), config.get_file_watcher());

    log::trace!("Starting UI...");
    UI::new(
        Layout::from_str(&config.get_layout(), todo.clone(), &config)?,
        todo.clone(),
        tx.clone(),
        &config,
    )
    .run()?;

    tx.send(FileWorkerCommands::Exit).unwrap();
    Ok(())
}
