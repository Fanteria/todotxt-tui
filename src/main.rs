use std::error::Error;
use todo_tui::{
    config::{Config, Logger},
    ui::UI,
};

fn main() {
    let config = Config::new();
    let run = || -> Result<(), Box<dyn Error>> {
        if !config.export()? {
            Logger::new(&config).init()?;
            log::trace!("===== START LOGGING =====");
            let mut ui = UI::build(&config)?;
            log::trace!("===== STARING UI =====");
            ui.run()?;
        }
        Ok(())
    };
    if let Err(e) = run() {
        eprintln!("{}", e);
    }
}
