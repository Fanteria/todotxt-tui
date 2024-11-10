use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config as LogConfig,
};
use std::{env, error::Error, io::stdin, path::PathBuf, process::exit};
use todotxt_tui::{
    config::{ConfMerge, Config},
    ui::UI,
    ToDoError,
};

/// Initializes the logging system.
///
/// Reads the log configuration file path from the environment variable.
/// If it is not set, the default path `config_folder/log4rs.yaml` is used.
fn log_init() -> Result<(), Box<dyn Error>> {
    let config_folder = Config::config_folder();
    let log_file = match env::var(format!(
        "{}LOGCONFIG",
        format_args!(
            "{}_",
            env!("CARGO_PKG_NAME").to_uppercase().replace('-', "_")
        )
    )) {
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

fn ask_to_create_config(err: ToDoError) -> ToDoError {
    fn ask() -> bool {
        let mut s = String::new();
        println!("Do you want to initialize it with default configuration? [y/N]");
        stdin()
            .read_line(&mut s)
            .expect("Did not enter a correct string");
        match s.trim().to_lowercase().as_str() {
            "y" | "yes" => true,
            "n" | "no" | "" => false,
            _ => {
                println!("You must say y or n");
                ask()
            }
        }
    }

    if let ToDoError::IOoperationFailed(file, error) = &err {
        if std::io::ErrorKind::NotFound == error.kind() {
            println!("Configuration file: {} does not exists.", file.display());
            if ask() {
                if let Err(e) = Config::export_default(file) {
                    return e;
                }
                println!("Configuration exported, please update todo_path.");
                exit(0);
            } else {
                exit(1);
            }
        }
    }
    err
}

fn main() {
    let run = || -> Result<(), Box<dyn Error>> {
        log_init()?;
        log::trace!("===== START LOGGING =====");
        let config = Config::from_args(env::args()).map_err(ask_to_create_config)?;
        let mut ui = UI::build(&config)?;
        log::trace!("===== STARTING UI =====");
        ui.run()?;
        Ok(())
    };
    if let Err(e) = run() {
        eprintln!("{}", e);
        exit(1);
    }
}
