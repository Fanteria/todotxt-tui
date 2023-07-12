use crate::todo::ToDo;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Result as ioResult, Write};
use std::str::FromStr;
use todo_txt::Task;

pub struct FileWorker {
    todo_path: String,
    archive_path: Option<String>,
}

impl FileWorker {
    pub fn new(todo_path: String, archive_path: Option<String>) -> FileWorker {
        FileWorker {
            todo_path,
            archive_path,
        }
    }

    pub fn load(&self, todo: &mut ToDo) -> ioResult<()> {
        Self::load_tasks(File::open(&self.todo_path)?, todo)?;
        if let Some(path) = &self.archive_path {
            Self::load_tasks(File::open(path)?, todo)?;
        }
        Ok(())
    }

    fn load_tasks<R: Read>(reader: R, todo: &mut ToDo) -> ioResult<()> {
        for line in BufReader::new(reader).lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match Task::from_str(line) {
                Ok(task) => todo.add_task(task),
                Err(_) => {} // TODO log or something
            }
        }
        Ok(())
    }

    pub fn save(&self, todo: &ToDo) -> ioResult<()> {
        let mut f = File::create(&self.todo_path)?;
        Self::save_tasks(&mut f, &todo.pending)?;
        match &self.archive_path {
            Some(s) => Self::save_tasks(&mut File::create(s)?, &todo.pending),
            None => Self::save_tasks(&mut f, &todo.done),
        }
    }

    fn save_tasks<W: Write>(writer: &mut W, tasks: &Vec<Task>) -> ioResult<()> {
        let mut writer = BufWriter::new(writer);
        for task in tasks.iter() {
            writer.write_all(task.to_string().as_bytes())?;
        }
        Ok(())
    }
}
