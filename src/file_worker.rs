use crate::{
    config::{FileWorkerConfig, SavePolicy},
    error::Result,
    todo::ToDo,
    ToDoError,
};
use notify::{
    event::{AccessKind, AccessMode, EventKind, RemoveKind},
    Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::{
    error,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    result,
    str::FromStr,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use todo_txt::Task;

/// Commands that can be sent to the `FileWorker` for various file-related operations.
pub enum FileWorkerCommands {
    ForceSave,
    Save,
    Load,
    Exit,
}

/// Manages file operations for the todo list and archive.
pub struct FileWorker {
    config: FileWorkerConfig,
    todo: Arc<Mutex<ToDo>>,
}

impl FileWorker {
    /// Creates a new `FileWorker` instance.
    ///
    /// `todo_path` and `archive_path` are expanded using `shellexpand` to resolve environment
    /// variables. If path is non UTF-8, then expansion is not performed.
    pub fn new(mut config: FileWorkerConfig, todo: Arc<Mutex<ToDo>>) -> Result<FileWorker> {
        if let Some(todo_path) = config.todo_path.to_str() {
            config.todo_path = PathBuf::from(
                shellexpand::env(todo_path)
                    .map_err(|e| ToDoError::path_exapand(&config.todo_path, e))?
                    .to_string(),
            );
        }
        if let Some(archive_path) = &config.archive_path {
            if let Some(archive_path) = archive_path.to_str() {
                config.archive_path = Some(PathBuf::from(
                    shellexpand::env(archive_path)
                        .map_err(|e| ToDoError::path_exapand(&config.todo_path, e))?
                        .to_string(),
                ));
            }
        }
        log::info!(
            "Init file worker: file: {:?}, archive: {:?}",
            config.todo_path,
            config.archive_path
        );
        Ok(FileWorker { config, todo })
    }

    /// Loads todo list data from the file(s).
    ///
    /// This method loads data from the main todo list file and optionally from an archive file.
    pub fn load(&self) -> Result<()> {
        let mut todo = ToDo::default(); // TODO this can be improved
        Self::load_tasks(
            File::open(&self.config.todo_path)
                .map_err(|e| ToDoError::io_operation_failed(&self.config.todo_path, e))?,
            &mut todo,
        )?;
        log::info!(
            "Load tasks from file {}",
            self.config.todo_path.to_string_lossy()
        );
        if let Some(path) = &self.config.archive_path {
            log::info!("Load tasks from achive file {}", path.to_string_lossy());
            Self::load_tasks(
                File::open(path).map_err(|e| ToDoError::io_operation_failed(path, e))?,
                &mut todo,
            )?;
        }
        log::debug!("Loaded pending {}x tasks", todo.pending.len());
        log::debug!("Loaded done {}x tasks", todo.done.len());
        self.todo.lock().unwrap().move_data(todo);
        Ok(())
    }

    /// Loads tasks from a given reader and adds them to the provided `ToDo` instance.
    fn load_tasks<R: Read>(reader: R, todo: &mut ToDo) -> Result<()> {
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

    /// Saves todo list data to the file(s).
    ///
    /// This method saves data to the main todo list file and optionally to an archive file.
    fn save(&self) -> Result<()> {
        let mut f = File::create(&self.config.todo_path)
            .map_err(|e| ToDoError::io_operation_failed(&self.config.todo_path, e))?;
        let todo = self.todo.lock().unwrap();
        log::info!(
            "Saving todo task to {}{}",
            self.config.todo_path.to_string_lossy(),
            self.config
                .archive_path
                .as_ref()
                .map_or(String::from(""), |p| String::from(" and")
                    + &p.to_string_lossy()),
        );
        Self::save_tasks(&mut f, &todo.pending)?;
        match &self.config.archive_path {
            Some(s) => Self::save_tasks(
                &mut File::create(s).map_err(|err| ToDoError::io_operation_failed(s, err))?,
                &todo.done,
            ),
            None => Self::save_tasks(&mut f, &todo.done),
        }
    }

    /// Saves a list of tasks to the provided writer.
    fn save_tasks<W: Write>(writer: &mut W, tasks: &[Task]) -> Result<()> {
        let mut writer = BufWriter::new(writer);
        for task in tasks.iter() {
            writer.write_all((task.to_string() + "\n").as_bytes())?;
        }
        Ok(())
    }

    /// Runs the `FileWorker` thread.
    ///
    /// This method starts the `FileWorker` thread and handles file-related operations and
    /// synchronization with other parts of the application.
    pub fn run(self) -> Result<Sender<FileWorkerCommands>> {
        use FileWorkerCommands::*;
        let (tx, rx) = mpsc::channel::<FileWorkerCommands>();

        match self.config.save_policy {
            SavePolicy::AutoSave if !self.config.autosave_duration.is_zero() => {
                Self::spawn_autosave(tx.clone(), self.config.autosave_duration);
            }
            SavePolicy::OnChange => {
                self.todo.lock().unwrap().get_version_mut().tx = Some(tx.clone());
            }
            _ => {}
        }

        if self.config.file_watcher {
            Self::spawn_watcher(tx.clone(), &self.config.todo_path)?;
            if let Some(path) = &self.config.archive_path {
                Self::spawn_watcher(tx.clone(), path)?;
            }
        }

        thread::spawn(move || {
            let mut versions = self.todo.lock().unwrap().get_version().get_version_all();
            let mut save_queue = 0;
            for received in rx {
                log::debug!("Save queue value: {save_queue}");
                if let Err(e) = match received {
                    Save => {
                        log::trace!("Try to save Todo list.");
                        if self
                            .todo
                            .lock()
                            .unwrap()
                            .get_version()
                            .is_actual_all(versions)
                        {
                            log::info!("File Worker: Todo list is actual.");
                            Ok(())
                        } else {
                            save_queue += 1;
                            if self.config.archive_path.is_some() {
                                save_queue += 1;
                            }
                            self.save()
                        }
                    }
                    ForceSave => {
                        log::trace!("Force save Todo list.");
                        save_queue += 1;
                        if self.config.archive_path.is_some() {
                            save_queue += 1;
                        }
                        let result = self.save();
                        versions = self.todo.lock().unwrap().get_version().get_version_all();
                        result
                    }
                    Load => {
                        if save_queue == 0 {
                            let result = self.load();
                            versions = self.todo.lock().unwrap().get_version().get_version_all();
                            log::info!("Todo list updated from file.");
                            result
                        } else {
                            save_queue -= 1;
                            Ok(())
                        }
                    }
                    Exit => break,
                } {
                    log::error!("File Worker: {}", e);
                }
            }
        });
        Ok(tx)
    }

    /// Spawns an autosave thread that periodically saves the todo list data.
    fn spawn_autosave(tx: Sender<FileWorkerCommands>, duration: Duration) {
        log::trace!("Start autosaver");
        thread::spawn(move || loop {
            thread::sleep(duration);
            log::trace!("Autosave with duration {}", duration.as_secs_f64());
            if tx.send(FileWorkerCommands::Save).is_err() {
                log::trace!("Autosave end");
            }
        });
    }

    /// Spawns a file watcher thread to monitor changes to a specific file.
    fn spawn_watcher(tx: Sender<FileWorkerCommands>, path: &Path) -> Result<()> {
        log::trace!("Start file watcher");
        let (tx_handle, rx_handle) = std::sync::mpsc::channel();
        let mut watcher: RecommendedWatcher = Watcher::new(tx_handle, NotifyConfig::default())?;
        watcher.watch(path, RecursiveMode::NonRecursive)?;
        thread::spawn(move || {
            for res in rx_handle {
                use EventKind::*;
                match res {
                    Ok(event) => match event.kind {
                        Access(AccessKind::Close(AccessMode::Write)) => {
                            for p in event.paths {
                                log::trace!("File {} changed", p.to_string_lossy());
                                if let Err(e) = tx.send(FileWorkerCommands::Load) {
                                    log::error!("Failed to load file {e}");
                                    return;
                                };
                            }
                        }
                        Remove(RemoveKind::File) => {
                            if let Err(e) = event.paths.into_iter().try_for_each(
                                |p| -> result::Result<(), Box<dyn error::Error>> {
                                    watcher.watch(&p, RecursiveMode::NonRecursive)?;
                                    log::debug!("Rename file {p:?}");
                                    tx.send(FileWorkerCommands::Load)?;
                                    Ok(())
                                },
                            ) {
                                log::error!("Failed to load file {e}");
                                return;
                            }
                        }
                        _ => log::debug!("Change: kind={:?}, paths={:?}", event.kind, event.paths),
                    },
                    Err(error) => log::error!("Error: {error:?}"),
                }
            }
            drop(watcher);
        });
        Ok(())
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
    fn test_load_tasks() -> Result<()> {
        let mut todo = ToDo::default();
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
    fn test_write_tasks() -> Result<()> {
        let mut todo = ToDo::default();
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
        let pretty_assert = |tasks, expected: &str, msg: &str| -> Result<()> {
            let mut buf: Vec<u8> = Vec::new();
            FileWorker::save_tasks(&mut buf, tasks)?;
            assert_eq!(
                expected.as_bytes(),
                buf,
                // if test failed print data in string not only in byte array
                "\n-----{}-----\nGET:\n{}\n----------------\nEXPECTED:\n{}\n",
                msg,
                String::from_utf8(buf.clone()).unwrap(),
                expected
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

    #[test]
    fn new_expand_env() -> Result<()> {
        let config = FileWorkerConfig {
            todo_path: PathBuf::from("$TODO_TUI_TEST_DIR/test"),
            archive_path: Some(PathBuf::from("$XDG_CONFIG_HOME/test")),
            ..Default::default()
        };

        let file_worker = FileWorker::new(config, Arc::new(Mutex::new(ToDo::default())))?;
        assert_eq!(
            file_worker.config.todo_path,
            PathBuf::from(env!("TODO_TUI_TEST_DIR").to_string() + "/test")
        );
        assert_eq!(
            file_worker.config.archive_path,
            Some(PathBuf::from(env!("XDG_CONFIG_HOME").to_string() + "/test"))
        );

        Ok(())
    }
}
