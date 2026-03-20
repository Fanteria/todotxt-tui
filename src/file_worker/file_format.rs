mod ical;
mod todotxt;

use crate::{file_worker::file_format::ical::ICal, todo::ToDo};
use anyhow::Result;

pub trait FileFormatTrait: Send {
    fn load_tasks(&self, todo: &mut ToDo) -> Result<()>;
    fn save_tasks(&self, todo: &ToDo) -> Result<()>;
}

pub fn new_file_format() -> Result<Box<dyn FileFormatTrait>> {
    if path.is_dir() {
        return Box::new(ICal::new());
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some("ics" | "ical") => FileFormat::ICal,
        _ => FileFormat::TodoTxt,
    }
    todo!()
}
