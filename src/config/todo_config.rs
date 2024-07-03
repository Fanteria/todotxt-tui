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

#[derive(Serialize, Deserialize, Parser, Debug, PartialEq, Eq, Clone)]
pub struct ToDoConfig {
    #[arg(long, default_value_t = default_use_done())]
    #[serde(default = "default_use_done")]
    pub use_done: bool,

    /// Sorting option for pending tasks.
    #[arg(long, value_name = "TASK_SORT", default_value_t)]
    #[serde(default)]
    pub pending_sort: TaskSort,

    /// Sorting option for completed tasks.
    #[arg(long, value_name = "TASK_SORT", default_value_t)]
    #[serde(default)]
    pub done_sort: TaskSort,

    #[arg(long, default_value_t = default_delete_final_date())]
    #[serde(default = "default_delete_final_date")]
    pub delete_final_date: bool,

    #[arg(long, default_value_t)]
    #[serde(default)]
    pub set_final_date: SetFinalDateType,
}

impl Default for ToDoConfig {
    fn default() -> Self {
        Self {
            use_done: default_use_done(),
            pending_sort: TaskSort::default(),
            done_sort: TaskSort::default(),
            delete_final_date: default_delete_final_date(), 
            set_final_date: SetFinalDateType::default(),
        }
    }
}

fn default_use_done() -> bool {
    false
}

fn default_delete_final_date() -> bool {
    true
}
