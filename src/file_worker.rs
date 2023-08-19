use crate::todo::ToDo;
use notify::{event::{EventKind, AccessKind, AccessMode}, Config, RecommendedWatcher, RecursiveMode, Watcher, recommended_watcher};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Result as ioResult, Write};
use std::path::Path;
use std::str::FromStr;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex};
use std::{thread, time::Duration};
use todo_txt::Task;

pub enum FileWorkerCommands {
    ForceSave,
    Save,
    Load,
    Exit,
}

pub struct FileWorker {
    todo_path: String,
    archive_path: Option<String>,
    todo: Arc<Mutex<ToDo>>,
}

impl FileWorker {
    pub fn new(
        todo_path: String,
        archive_path: Option<String>,
        todo: Arc<Mutex<ToDo>>,
    ) -> FileWorker {
        FileWorker {
            todo_path,
            archive_path,
            todo,
        }
    }

    fn load(&self) -> ioResult<()> {
        let mut todo = ToDo::new(false);
        Self::load_tasks(File::open(&self.todo_path)?, &mut todo)?;
        if let Some(path) = &self.archive_path {
            Self::load_tasks(File::open(path)?, &mut todo)?;
        }
        self.todo.lock().unwrap().move_data(todo);
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
                Err(e) => log::warn!("Task cannot be load due {e}: {line}"),
            }
        }
        Ok(())
    }

    fn save(&self) -> ioResult<()> {
        let mut f = File::create(&self.todo_path)?;
        let todo = self.todo.lock().unwrap();
        log::info!(
            "Saveing todo task to {}{}",
            self.todo_path,
            self.archive_path
                .as_ref()
                .map_or(String::from(""), |p| String::from(" and") + &p.clone()),
        );
        Self::save_tasks(&mut f, &todo.pending)?;
        match &self.archive_path {
            Some(s) => Self::save_tasks(&mut File::create(s)?, &todo.pending),
            None => Self::save_tasks(&mut f, &todo.done),
        }
    }

    fn save_tasks<W: Write>(writer: &mut W, tasks: &[Task]) -> ioResult<()> {
        let mut writer = BufWriter::new(writer);
        for task in tasks.iter() {
            writer.write_all((task.to_string() + "\n").as_bytes())?;
        }
        Ok(())
    }

    pub fn run(
        self,
        autosave_duration: Duration,
        handle_changes: bool,
    ) -> Sender<FileWorkerCommands> {
        use FileWorkerCommands::*;
        let (tx, rx) = mpsc::channel::<FileWorkerCommands>();
        if !autosave_duration.is_zero() {
            let tx_worker = tx.clone();
            log::trace!("Start autosaver");
            thread::spawn(move || loop {
                thread::sleep(autosave_duration);
                log::trace!("Autosave with duration {}", autosave_duration.as_secs_f64());
                if tx_worker.send(Save).is_err() {
                    log::trace!("Autosave end");
                }
            });
        }

        if handle_changes {
            let tx_worker = tx.clone();
            log::trace!("Start file watcher");
            thread::spawn(move || loop {
                let (tx_handle, rx_handle) = std::sync::mpsc::channel();
                let mut watcher = RecommendedWatcher::new(tx_handle, Config::default()).unwrap();
                watcher
                    .watch(
                        Path::new("/home/jirka/todo.txt"),
                        RecursiveMode::NonRecursive,
                    )
                    .unwrap();
                for res in rx_handle {
                    match res {
                        Ok(event) => {
                            match event.kind {
                                EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                                    log::warn!("Change:")
                                }
                                _ => log::debug!("Change: {event:?}"),
                            }
                        }
                        Err(error) => log::error!("Error: {error:?}"),
                    }
                }
            });
        }

        thread::spawn(move || {
            let mut version = self.todo.lock().unwrap().get_version();
            for received in rx {
                if let Err(e) = match received {
                    Save => {
                        let act_version = self.todo.lock().unwrap().get_version();
                        if version == act_version {
                            log::trace!("File Worker: Todo list is actual.");
                            Ok(())
                        } else {
                            version = act_version;
                            self.save()
                        }
                    }
                    ForceSave => self.save(),
                    Load => {
                        let result = self.load();
                        version = self.todo.lock().unwrap().get_version();
                        result
                    }
                    Exit => break,
                } {
                    log::error!("File Worker: {}", e.kind());
                }
            }
        });
        tx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TESTING_STRING: &str = r#"
        x (A) 2023-05-21 2023-04-30 measure space for 1 +project1 @context1 #hashtag1 due:2023-06-30
                         2023-04-30 measure space for 2 +project2 @context2           due:2023-06-30
                     (C) 2023-04-30 measure space for 3 +project3 @context3           due:2023-06-30
                                    measure space for 4 +project2 @context3 #hashtag1 due:2023-06-30
                                  x measure space for 5 +project3 @context3 #hashtag2 due:2023-06-30
                                    measure space for 6 +project3 @context2 #hashtag2 due:2023-06-30
        "#;

    #[test]
    fn test_load_tasks() -> ioResult<()> {
        let mut todo = ToDo::new(false);
        FileWorker::load_tasks(TESTING_STRING.as_bytes(), &mut todo)?;
        assert_eq!(todo.pending.len(), 4);
        assert_eq!(todo.done.len(), 2);
        assert_eq!(
            todo.pending[0].subject,
            "measure space for 2 +project2 @context2"
        );
        assert_eq!(
            todo.pending[1].subject,
            "measure space for 3 +project3 @context3"
        );
        assert_eq!(todo.pending[1].priority, 2);
        assert_eq!(
            todo.pending[2].subject,
            "measure space for 4 +project2 @context3 #hashtag1"
        );
        assert_eq!(
            todo.pending[3].subject,
            "measure space for 6 +project3 @context2 #hashtag2"
        );

        assert_eq!(
            todo.done[0].subject,
            "measure space for 1 +project1 @context1 #hashtag1"
        );
        assert_eq!(
            todo.done[1].subject,
            "measure space for 5 +project3 @context3 #hashtag2"
        );

        Ok(())
    }

    #[test]
    fn test_write_tasks() -> ioResult<()> {
        let mut todo = ToDo::new(false);
        FileWorker::load_tasks(TESTING_STRING.as_bytes(), &mut todo)?;
        let get_expected = |line: fn(&String) -> bool| {
            TESTING_STRING
                .trim()
                .lines()
                .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
                .filter(line)
                .collect::<Vec<String>>()
                .join("\n")
                + "\n"
        };
        let pretty_assert = |tasks, expected: &str, msg: &str| -> ioResult<()> {
            let mut buf: Vec<u8> = Vec::new();
            FileWorker::save_tasks(&mut buf, tasks)?;
            assert_eq!(
                expected.as_bytes(),
                buf,
                // if test failed print data in string not only in byte array
                "\n-----{}-----\nGET:\n{}\n----------------\nEXPECTED:\n{}\n",
                msg,
                String::from_utf8(buf.clone()).unwrap(),
                expected.clone()
            );
            Ok(())
        };

        pretty_assert(
            &todo.pending,
            &get_expected(|line| !line.starts_with("x ")),
            "Pending check is wrong",
        )?;
        pretty_assert(
            &todo.done,
            &get_expected(|line| line.starts_with("x ")),
            "Done check is wrong",
        )?;

        Ok(())
    }
}
