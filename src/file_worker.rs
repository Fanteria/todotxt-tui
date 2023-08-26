use crate::todo::ToDo;
use notify::{
    event::{AccessKind, AccessMode, EventKind},
    Config, RecommendedWatcher, RecursiveMode, Watcher,
};
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
        log::info!("Init file worker: file: {}, archive: {:?}", todo_path, archive_path);
        FileWorker {
            todo_path,
            archive_path,
            todo,
        }
    }

    pub fn load(&self) -> ioResult<()> {
        let mut todo = ToDo::new(false);
        Self::load_tasks(File::open(&self.todo_path)?, &mut todo)?;
        log::info!("Load tasks from file {}", self.todo_path);
        if let Some(path) = &self.archive_path {
            log::info!("Load tasks from achive file {}", path);
            Self::load_tasks(File::open(path)?, &mut todo)?;
        }
        log::debug!("Loaded pending {}x tasks", todo.pending.len());
        log::debug!("Loaded done {}x tasks", todo.done.len());
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
            Some(s) => Self::save_tasks(&mut File::create(s)?, &todo.done),
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
            Self::spawn_autosave(tx.clone(), autosave_duration);
        }

        if handle_changes {
            Self::spawn_watcher(tx.clone(), self.todo_path.clone());
            // TODO watcher update only one file
            // if let Some(path) = &self.archive_path {
            //     Self::spawn_watcher(tx.clone(), path.clone());
            // }
        }

        thread::spawn(move || {
            let mut version = self.todo.lock().unwrap().get_version();
            let mut load_skip = false;
            for received in rx {
                if let Err(e) = match received {
                    Save => {
                        load_skip = true;
                        let act_version = self.todo.lock().unwrap().get_version();
                        if version == act_version {
                            log::trace!("File Worker: Todo list is actual.");
                            Ok(())
                        } else {
                            version = act_version;
                            self.save()
                        }
                    }
                    ForceSave => {
                        load_skip = true;
                        self.save()
                    }
                    Load => {
                        if load_skip {
                            load_skip = false;
                            continue;
                        }
                        let result = self.load();
                        version = self.todo.lock().unwrap().get_version();
                        log::info!("Todo list updated from file.");
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

    fn spawn_autosave(tx: Sender<FileWorkerCommands>, duration: Duration) {
        let tx_worker = tx.clone();
        log::trace!("Start autosaver");
        thread::spawn(move || loop {
            thread::sleep(duration);
            log::trace!("Autosave with duration {}", duration.as_secs_f64());
            if tx_worker.send(FileWorkerCommands::Save).is_err() {
                log::trace!("Autosave end");
            }
        });
    }

    fn spawn_watcher(tx: Sender<FileWorkerCommands>, path: String) {
        log::trace!("Start file watcher");
        thread::spawn(move || {
            let (tx_handle, rx_handle) = std::sync::mpsc::channel();
            let mut watcher: RecommendedWatcher =
                Watcher::new(tx_handle, Config::default()).unwrap();
            watcher
                .watch(Path::new(&path), RecursiveMode::NonRecursive)
                .unwrap();
            for res in rx_handle {
                match res {
                    Ok(event) => match event.kind {
                        EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                            log::trace!("File {} changed", path);
                            if tx.send(FileWorkerCommands::Load).is_err() {
                                break;
                            };
                        }
                        _ => log::debug!("Change: {event:?}"),
                    },
                    Err(error) => log::error!("Error: {error:?}"),
                }
            }
        });
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
