use std::{
    env,
    ffi::OsString,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use clap::builder::Styles;

use crate::{ToDoIoError, ToDoRes};

pub trait Conf: Sized + Default {
    fn from_file(path: impl AsRef<Path>) -> ToDoRes<Self> {
        Ok(Self::from_reader(
            File::open(path.as_ref()).map_err(|e| ToDoIoError::new(path.as_ref(), e))?,
        )?)
    }

    fn from_reader<R>(reader: R) -> ToDoRes<Self>
    where
        R: Read;

    fn parse<Iter, T, R>(iter: Iter, reader: R) -> ToDoRes<Self>
    where
        Iter: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
        R: Read;
}

pub trait ConfMerge: Sized + ConfigDefaults + Conf {
    fn new() -> ToDoRes<Self> {
        Self::from_args(env::args())
    }

    fn from_args<Iter, T>(iter: Iter) -> ToDoRes<Self>
    where
        Iter: IntoIterator<Item = T>,
        T: Into<OsString> + Clone;
}

pub trait ConfigDefaults {
    fn config_path() -> PathBuf;

    fn env_prefix() -> String;

    fn help_colors() -> Styles {
        Styles::plain()
    }
}
