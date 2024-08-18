use std::fmt::Display;

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, ValueEnum, Clone, Debug, PartialEq, Eq, Default)]
pub enum SetFinalDateType {
    Override,
    #[default]
    OnlyMissing,
    Never,
}

impl Display for SetFinalDateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&super::parsers::enum_debug_display_parser(&format!("{:?}", self)))?;
        Ok(())
    }
}

/// Represents the possible sorting options for tasks.
#[derive(Clone, Copy, Serialize, Deserialize, Default, ValueEnum, Debug, PartialEq, Eq)]
pub enum TaskSort {
    #[default]
    None,
    Reverse,
    Priority,
    Alphanumeric,
    AlphanumericReverse,
}

impl Display for TaskSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&super::parsers::enum_debug_display_parser(&format!("{:?}", self)))?;
        Ok(())
    }
}

