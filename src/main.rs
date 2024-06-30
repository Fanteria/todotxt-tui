use std::error::Error;
use todotxt_tui::{config::Commands, config::Config, ui::UI};

fn main() {
    let config = Config::new();
    let run = || -> Result<(), Box<dyn Error>> {
        match &config.command {
            Some(Commands::Run) | None => {
                config.logger.init()?;
                log::trace!("===== START LOGGING =====");
                let mut ui = UI::build(&config)?;
                log::trace!("===== STARING UI =====");
                ui.run()?;
            }
            Some(Commands::Autocomplete { path }) => Config::generate_autocomplete(path)?,
            Some(Commands::Config { path }) => config.export_config(path)?,
            Some(Commands::DefaultConfig { path }) => Config::export_default_config(path)?,
        }
        Ok(())
    };
    if let Err(e) = run() {
        eprintln!("{}", e);
    }
}
