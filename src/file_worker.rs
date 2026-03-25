mod file_format;

use crate::{
    config::{FileWorkerConfig, SavePolicy},
    file_worker::file_format::{new_file_format, FileFormatTrait},
    todo::ToDo,
};
use anyhow::{Context, Result};
use notify::{
    event::{AccessKind, AccessMode, EventKind, RemoveKind},
    Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::{
    error,
    path::{Path, PathBuf},
    result,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

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
    format: Box<dyn FileFormatTrait>,
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
                    .with_context(|| format!("Failed to expand path {:?}", config.todo_path))?
                    .to_string(),
            );
        }
        if let Some(archive_path) = &config.archive_path {
            if let Some(archive_path) = archive_path.to_str() {
                config.archive_path = Some(PathBuf::from(
                    shellexpand::env(archive_path)
                        .with_context(|| {
                            format!("Failed to expand path {:?}", config.archive_path)
                        })?
                        .to_string(),
                ));
            }
        }
        log::info!(
            "Init file worker: file: {:?}, archive: {:?}",
            config.todo_path,
            config.archive_path
        );
        Ok(FileWorker {
            format: new_file_format(&config)?,
            config,
            todo,
        })
    }

    /// Loads todo list data from the file(s).
    ///
    /// This method loads data from the main todo list file and optionally from an archive file.
    pub fn load(&self) -> Result<()> {
        let mut todo = ToDo::default(); // TODO this can be improved
        self.format
            // TODO
            .load_tasks(&mut todo)
            .with_context(|| format!("{:?}", self.config.todo_path))?;
        log::info!(
            "Load tasks from {}",
            self.config.todo_path.to_string_lossy()
        );
        if let Some(path) = &self.config.archive_path {
            log::info!("Load tasks from archive {}", path.to_string_lossy());
            // TODO
            self.format
                .load_tasks(&mut todo)
                .with_context(|| format!("{path:?}"))?;
        }
        log::debug!("Loaded pending {}x tasks", todo.pending.len());
        log::debug!("Loaded done {}x tasks", todo.done.len());
        self.todo.lock().unwrap().move_data(todo);
        Ok(())
    }

    /// Saves todo list data to the file(s).
    ///
    /// This method saves data to the main todo list file and optionally to an archive file.
    /// When no archive is configured, all tasks (pending + done) are saved together so that
    /// directory-based formats (iCalendar/vdirsyncer) can correctly manage orphan cleanup.
    fn save(&self) -> Result<()> {
        let todo = self.todo.lock().unwrap();
        log::info!(
            "Saving todo tasks to {}{}",
            self.config.todo_path.to_string_lossy(),
            self.config
                .archive_path
                .as_ref()
                .map_or(String::from(""), |p| String::from(" and ")
                    + &p.to_string_lossy()),
        );
        self.format.save_tasks(&todo)?;
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

    #[test]
    fn format_mismatch_rejected() {
        let config = FileWorkerConfig {
            todo_path: PathBuf::from("todo.ics"),
            archive_path: Some(PathBuf::from("archive.txt")),
            ..Default::default()
        };
        let result = FileWorker::new(config, Arc::new(Mutex::new(ToDo::default())));
        assert!(result.is_err());
    }
}
