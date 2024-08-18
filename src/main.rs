use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config as LogConfig,
};
use std::{env, error::Error, path::PathBuf, process::exit};
use todotxt_tui::{
    config::{ConfMerge, Config, Conf},
    ui::UI,
};

fn log_init() -> Result<(), Box<dyn Error>> {
    let config_folder = Config::config_folder();
    let log_file = match env::var(format!("{}LOGCONFIG", Config::env_prefix())) {
        Ok(log_file) => PathBuf::from(log_file),
        Err(_) => config_folder.join("log4rs.yaml"),
    };
    if log_file.exists() {
        log4rs::init_file(log_file, Default::default())?;
    } else {
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} [{h({l})}] {M}: {m}{n}")))
            .build(config_folder.join("log.log"))?;
        let log_cofnig = LogConfig::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;
        log4rs::init_config(log_cofnig)?;
    }
    Ok(())
}

fn main() {
    let run = || -> Result<(), Box<dyn Error>> {
        log_init()?;
        log::trace!("===== START LOGGING =====");
        let config = Config::new()?;
        // TODO move logging to initialization and add log about loaded config file
        let mut ui = UI::build(&config)?;
        log::trace!("===== STARTING UI =====");
        ui.run()?;
        // }
        Ok(())
    };
    if let Err(e) = run() {
        eprintln!("{}", e);
        exit(1);
    }
}
