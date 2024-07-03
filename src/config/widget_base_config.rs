use clap::Parser;
use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};

use crate::ui::{EventHandlerUI, UIEvent};

#[derive(Serialize, Deserialize, Parser, Debug, PartialEq, Eq, Clone)]
pub struct WidgetBaseConfig {
    /// Task keybindings.
    #[clap(skip)]
    #[serde(default = "default_tasks_keybind")]
    pub tasks_keybind: EventHandlerUI,

    /// Category keybindings.
    #[clap(skip)]
    #[serde(default = "default_category_keybind")]
    pub category_keybind: EventHandlerUI,
}

impl Default for WidgetBaseConfig {
    fn default() -> Self {
        Self {
            tasks_keybind: default_tasks_keybind(),
            category_keybind: default_category_keybind(),
        }
    }
}

fn default_tasks_keybind() -> EventHandlerUI {
    EventHandlerUI::new(&[
        (KeyCode::Char('U'), UIEvent::SwapUpItem),
        (KeyCode::Char('D'), UIEvent::SwapDownItem),
        (KeyCode::Char('x'), UIEvent::RemoveItem),
        (KeyCode::Char('d'), UIEvent::MoveItem),
        (KeyCode::Enter, UIEvent::Select),
    ])
}

fn default_category_keybind() -> EventHandlerUI {
    EventHandlerUI::new(&[
        (KeyCode::Enter, UIEvent::Select),
        (KeyCode::Backspace, UIEvent::Remove),
    ])
}
