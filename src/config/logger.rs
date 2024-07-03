use clap::Parser;
use serde::{Deserialize, Serialize};
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config as LogConfig, Root},
    encode::pattern::PatternEncoder,
};
use std::{error::Error, path::PathBuf};

#[derive(Serialize, Deserialize, Parser, Debug, PartialEq, Eq, Clone)]
pub struct Logger {
    /// Path to the log file.
    #[arg(long, default_value = default_file().into_os_string(), value_name = "FILE")]
    #[serde(default = "default_file")]
    log_file: PathBuf,

    /// Log format (uses placeholders)
    #[arg(long, default_value_t = default_format())]
    #[serde(default = "default_format")]
    log_format: String,

    /// Log level.
    #[arg(long, default_value_t = default_level(), value_name = "LOG_LEVEL")]
    #[serde(default = "default_level")]
    log_level: LevelFilter,
}

impl Logger {
    pub fn init(&self) -> Result<(), Box<dyn Error>> {
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(&self.log_format)))
            .build(&self.log_file)?;
        let logging_config = LogConfig::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(Root::builder().appender("logfile").build(self.log_level))?;
        log4rs::init_config(logging_config)?;
        Ok(())
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            log_file: default_file(),
            log_format: default_format(),
            log_level: default_level(),
        }
    }
}

fn default_file() -> PathBuf {
    PathBuf::from("log.log")
}

fn default_format() -> String {
    String::from("{d} [{h({l})}] {M}: {m}{n}")
}

fn default_level() -> LevelFilter {
    LevelFilter::Info
}
