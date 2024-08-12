use crate::ui::{EventHandlerUI, UIEvent};

use clap::Parser;
use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Parser, Debug, PartialEq, Eq, Clone)]
pub struct ListConfig {
    /// Indentation level for lists.
    #[arg(short, long, default_value_t = default_list_shift())]
    #[serde(default = "default_list_shift")]
    pub list_shift: usize,

    /// List keybindings.
    // #[clap(skip)]
    #[arg(long, default_value_t = default_list_keybind())]
    #[serde(default = "default_list_keybind")]
    pub list_keybind: EventHandlerUI,
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            list_shift: default_list_shift(),
            list_keybind: default_list_keybind(),
        }
    }
}

fn default_list_shift() -> usize {
    4
}

fn default_list_keybind() -> EventHandlerUI {
    EventHandlerUI::new(&[
        (KeyCode::Char('j'), UIEvent::ListDown),
        (KeyCode::Char('k'), UIEvent::ListUp),
        (KeyCode::Char('g'), UIEvent::ListFirst),
        (KeyCode::Char('G'), UIEvent::ListLast),
    ])
}
