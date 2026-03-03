mod ical;
mod todotxt;

use crate::todo::ToDo;
use anyhow::Result;
use std::{
    io::{Read, Write},
    path::Path,
};
use todo_txt::task::Simple as Task;

pub enum FileFormat {
    TodoTxt,
    ICal,
}

impl FileFormat {
    /// Detect format from file extension.
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("ics" | "ical") => FileFormat::ICal,
            _ => FileFormat::TodoTxt,
        }
    }

    /// Returns true if both paths use the same format.
    pub fn matches(path_a: &Path, path_b: &Path) -> bool {
        matches!(
            (Self::from_path(path_a), Self::from_path(path_b)),
            (FileFormat::TodoTxt, FileFormat::TodoTxt) | (FileFormat::ICal, FileFormat::ICal)
        )
    }

    pub fn load_tasks<R: Read>(&self, reader: R, todo: &mut ToDo) -> Result<()> {
        match self {
            FileFormat::TodoTxt => todotxt::load_tasks(reader, todo),
            FileFormat::ICal => ical::load_tasks(reader, todo),
        }
    }

    pub fn save_tasks<W: Write>(&self, writer: &mut W, tasks: &[Task]) -> Result<()> {
        match self {
            FileFormat::TodoTxt => todotxt::save_tasks(writer, tasks),
            FileFormat::ICal => ical::save_tasks(writer, tasks),
        }
    }
}
