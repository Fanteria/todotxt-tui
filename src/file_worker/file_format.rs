mod ical;
mod todotxt;

use crate::todo::ToDo;
use anyhow::Result;
use std::path::Path;
use todo_txt::task::Simple as Task;

pub trait FileFormatTrait {
    fn load_tasks(&self, todo: &mut ToDo) -> Result<()>;
    fn save_tasks(&self, todo: &mut ToDo) -> Result<()>;
}

pub enum FileFormat {
    TodoTxt,
    ICal,
}

impl FileFormat {
    /// Detect format from path: directories are treated as iCalendar (vdirsyncer),
    /// files with .ics/.ical extension are also iCalendar, everything else is TodoTxt.
    pub fn from_path(path: &Path) -> Self {
        if path.is_dir() {
            return FileFormat::ICal;
        }
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

    pub fn load_tasks(&self, path: &Path, todo: &mut ToDo) -> Result<()> {
        match self {
            FileFormat::TodoTxt => todotxt::load_tasks(path, todo),
            FileFormat::ICal => ical::load_tasks(path, todo),
        }
    }

    pub fn save_tasks(&self, path: &Path, tasks: &[Task]) -> Result<()> {
        match self {
            FileFormat::TodoTxt => todotxt::save_tasks(path, tasks),
            FileFormat::ICal => ical::save_tasks(path, tasks),
        }
    }
}
