use clap::builder::Styles;
use std::{
    env,
    ffi::OsString,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use crate::{Result, ToDoIoError};

pub trait Conf: Sized + Default {
    fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_reader(
            File::open(path.as_ref()).map_err(|e| ToDoIoError::new(path.as_ref(), e))?,
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

    fn env_prefix() -> String {
        format!(
            "{}_",
            env!("CARGO_PKG_NAME").to_uppercase().replace('-', "_")
        )
    }
}

pub trait ConfMerge: Sized + ConfigDefaults + Conf {
    fn new() -> Result<Self> {
        Self::from_args(env::args())
    }

    fn from_args<Iter, T>(iter: Iter) -> Result<Self>
    where
        Iter: IntoIterator<Item = T>,
        T: Into<OsString> + Clone;
}

pub trait ConfigDefaults {
    fn config_path() -> PathBuf;

    fn help_colors() -> Styles {
        Styles::plain()
    }
}
