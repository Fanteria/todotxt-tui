use std::error::Error;
use todotxt_tui::{config::Config, ui::UI};

fn main() {
    let run = || -> Result<(), Box<dyn Error>> {
        let config = Config::new();
        if let Some(path) = &config.export_autocomplete {
            Config::generate_autocomplete(path)?;
        } else if let Some(path) = &config.export_default_config {
            Config::export_default_config(path)?;
        } else if let Some(path) = &config.export_config {
            config.export_config(path)?;
        } else {
            config.logger.init()?;
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
