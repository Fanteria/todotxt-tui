use super::TextStyle;

use clap::Parser;
use serde::{Deserialize, Serialize};
use super::colors::Color;


#[derive(Serialize, Deserialize, Parser, Debug, PartialEq, Eq, Clone)]
pub struct ActiveColorConfig {
    /// Color for the active list item.
    #[arg(long, value_name = "TEXT_STYLE", default_value_t = default_list_active_color())]
    #[serde(default = "default_list_active_color")]
    pub list_active_color: TextStyle,
    
    /// Color for active pending task.
    #[arg(long, value_name = "TEXT_STYLE", default_value_t)]
    #[serde(default)]
    pub pending_active_color: TextStyle,

    /// Color for active completed task.
    #[arg(long, value_name = "TEXT_STYLE", default_value_t)]
    #[serde(default)]
    pub done_active_color: TextStyle,
}

impl Default for ActiveColorConfig {
    fn default() -> Self {
        Self {
            list_active_color: default_list_active_color(),
            pending_active_color: TextStyle::default(),
            done_active_color: TextStyle::default(),
        }
    }
}

fn default_list_active_color() -> TextStyle {
    TextStyle::default().bg(Color::lightred())
}
