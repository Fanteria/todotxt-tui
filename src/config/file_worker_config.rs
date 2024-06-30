use std::{env::var, path::PathBuf, time::Duration};

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Parser, Debug, PartialEq, Eq, Clone)]
pub struct FileWorkerConfig {
    #[arg(short, long, default_value = default_todo_path().into_os_string(), value_name = "PATH")]
    #[serde(default = "default_todo_path")]
    pub todo_path: PathBuf,

    #[arg(short, long, value_name = "PATH")]
    #[serde(default)]
    pub archive_path: Option<PathBuf>,

    #[arg(short = 'd', long, default_value = default_autosave_duration().as_secs().to_string(), value_parser = super::parsers::parse_duration, value_name = "DURATION")]
    #[serde(default = "default_autosave_duration")]
    pub autosave_duration: Duration,

    #[arg(short, long, value_name = "FLAG")]
    #[serde(default = "default_file_watcher")]
    pub file_watcher: bool,
}

impl Default for FileWorkerConfig {
    fn default() -> Self {
        Self {
            todo_path: default_todo_path(),
            archive_path: None,
            autosave_duration: default_autosave_duration(),
            file_watcher: default_file_watcher(),
        }
    }
}

fn default_todo_path() -> PathBuf {
    PathBuf::from(var("HOME").unwrap_or(String::from("~")) + "/todo.txt")
}

fn default_autosave_duration() -> Duration {
    Duration::from_secs(900)
}

fn default_file_watcher() -> bool {
    true
}
