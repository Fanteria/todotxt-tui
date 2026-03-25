mod ical;
mod todotxt;

use crate::{
    config::FileWorkerConfig,
    file_worker::file_format::{
        ical::{ICal, ICalSingleFile},
        todotxt::TodoTxt,
    },
    todo::ToDo,
};
use anyhow::{anyhow, Result};

pub trait FileFormatTrait: Send {
    fn load_tasks(&self, todo: &mut ToDo) -> Result<()>;
    fn save_tasks(&self, todo: &ToDo) -> Result<()>;
}

pub fn new_file_format(config: &FileWorkerConfig) -> Result<Box<dyn FileFormatTrait>> {
    if config.todo_path.is_dir() {
        return match config.archive_path {
            Some(_) => Err(anyhow!(
                "archive_path is not supported for the iCalendar format"
            )),
            None => Ok(Box::new(ICal::new(config))),
        };
    }
    match config.todo_path.extension().and_then(|e| e.to_str()) {
        Some("ics" | "ical") if config.archive_path.is_none() => {
            Ok(Box::new(ICalSingleFile::new(config)))
        }
        Some("ics" | "ical") => Err(anyhow!(
            "archive_path is not supported for the iCalendar format"
        )),
        _ => Ok(Box::new(TodoTxt::new(config))),
    }
}
