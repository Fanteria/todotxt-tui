use super::{traits::ExportConf, ConfMerge, Config};
use crate::error::Result;
use clap::ArgMatches;
use std::{
    io::{stdout, Write},
    path::{Path, PathBuf},
};

#[derive(clap::Parser, Debug, PartialEq, Eq, Clone, Default)]
pub struct Export {
    /// Generate autocomplete script to given file path.
    /// If path is not set, standard output will be used.
    #[clap(long, group = "export", help_heading = "Export")]
    #[arg(value_name = "PATH")]
    pub export_autocomplete: Option<Option<std::path::PathBuf>>,

    /// Generate full configuration file for actual session
    /// so present configuration file and command lines
    /// options are taken in account. If path is not set,
    /// standard output will be used.
    #[clap(long, group = "export", help_heading = "Export")]
    #[arg(value_name = "PATH")]
    pub export_config: Option<Option<std::path::PathBuf>>,

    /// Generate configuration file with default values
    /// to given file path. If path is not set, standard
    /// output will be used.
    #[clap(long, group = "export", help_heading = "Export")]
    #[arg(value_name = "PATH")]
    pub export_default_config: Option<Option<std::path::PathBuf>>,
}

impl ExportConf for Export {
    fn export(&self, config_path: impl AsRef<Path>, matches: &ArgMatches) -> Result<()> {
        let create_writer = |path: Option<&PathBuf>| -> Result<Box<dyn Write>> {
            Ok(match path {
                Some(path) => Box::new(
                    std::fs::File::create(path)
                        .map_err(|e| crate::ToDoError::io_operation_failed(path, e))?,
                ),
                None => Box::new(stdout()),
            })
        };
        if let Some(path) = &self.export_autocomplete {
            Config::autocomplete(&mut create_writer(path.as_ref())?)?;
            std::process::exit(0);
        }
        if let Some(path) = &self.export_config {
            write!(
                create_writer(path.as_ref())?,
                "{}",
                Config::configured_toml(config_path, matches)?
            )?;
            std::process::exit(0);
        }
        if let Some(path) = &self.export_default_config {
            write!(create_writer(path.as_ref())?, "{}", Config::default_toml()?)?;
            std::process::exit(0);
        }
        Ok(())
    }
}
