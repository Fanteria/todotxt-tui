use crate::{Result, ToDoError};
use clap::{builder::Styles, ArgMatches};
use std::{
    ffi::OsString,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub trait Conf: Sized + Default {
    fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_reader(
            File::open(path.as_ref()).map_err(|e| ToDoError::io_operation_failed(path, e))?,
        )
    }

    fn from_reader<R>(reader: R) -> Result<Self>
    where
        R: Read;

    fn parse<Iter, T, R>(iter: Iter, reader: R) -> Result<Self>
    where
        Iter: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
        R: Read;
}

pub trait ConfMerge: Sized + ConfigDefaults + Conf {
    fn from_args<Iter, T>(iter: Iter) -> Result<Self>
    where
        Iter: IntoIterator<Item = T>,
        T: Into<OsString> + Clone;

    fn export_default(path: impl AsRef<Path>) -> Result<()> {
        let mut output = std::fs::File::create(path.as_ref())
            .map_err(|e| crate::ToDoError::io_operation_failed(path, e))?;
        write!(output, "{}", Self::default_toml()?)?;
        Ok(())
    }

    fn configured_toml(path: impl AsRef<Path>, matches: &ArgMatches) -> Result<String>;

    fn default_toml() -> Result<String>;

    fn autocomplete(writer: &mut impl std::io::Write) -> Result<()>;
}

pub trait ConfigDefaults {
    fn config_path() -> PathBuf;

    fn help_colors() -> Styles;
}
