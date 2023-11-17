use super::Config;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config as LogConfig, Root},
    encode::pattern::PatternEncoder,
};
use std::error::Error;

pub struct Logger {
    file: String,
    format: String,
    level: LevelFilter,
}

impl Logger {
    pub fn new(config: &Config) -> Self {
        Self {
            file: config.get_log_file(),
            format: config.get_log_format(),
            level: config.get_log_level(),
        }
    }

    pub fn init(&self) -> Result<(), Box<dyn Error>> {
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(&self.format)))
            .build(&self.file)?;
        let logging_config = LogConfig::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(Root::builder().appender("logfile").build(self.level))?;
        log4rs::init_config(logging_config)?;
        Ok(())
    }
}
