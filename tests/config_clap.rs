use std::error::Error;

use clap::Parser;
use todotxt_tui::config::Config;

#[test]
fn default_values() -> Result<(), Box<dyn Error>> {
    let default = Config::default();
    let default2 = Config::try_parse_from(Vec::<&str>::new())?;
    assert_eq!(default, default2);

    Ok(())
}

