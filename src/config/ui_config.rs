use crossterm::event::KeyCode;
use std::{path::PathBuf, time::Duration};

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::ui::{EventHandlerUI, UIEvent};

#[derive(Serialize, Deserialize, Parser, Debug, PartialEq, Eq, Clone)]
pub struct UiConfig {
    /// Title of window with opened todo-tui {env!("CARGO_PKG_NAME")} {AAAA} TODO
    #[arg(short = 'T', long, default_value_t = default_window_title(), value_name = "STRING")]
    #[serde(default = "default_window_title")]
    pub window_title: String,

    #[clap(skip)]
    #[serde(default = "default_window_keybinds")]
    pub window_keybinds: EventHandlerUI,

    #[arg(short = 'L', long, default_value = default_list_refresh_rate().as_secs().to_string(), value_parser = super::parsers::parse_duration, value_name = "DURATION")]
    #[serde(default = "default_list_refresh_rate")]
    pub list_refresh_rate: Duration,

    #[arg(long, value_name = "FILE", help_heading = "export")]
    #[serde(default)]
    pub save_state_path: Option<PathBuf>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            window_title: default_window_title(),
            window_keybinds: default_window_keybinds(),
            list_refresh_rate: default_list_refresh_rate(),
            save_state_path: None,
        }
    }
}

fn default_window_title() -> String {
    String::from("ToDo tui")
}

fn default_list_refresh_rate() -> Duration {
    Duration::from_secs(5)
}

fn default_window_keybinds() -> EventHandlerUI {
    EventHandlerUI::new(&[
        (KeyCode::Char('q'), UIEvent::Quit),
        (KeyCode::Char('S'), UIEvent::Save),
        (KeyCode::Char('u'), UIEvent::Load),
        (KeyCode::Char('H'), UIEvent::MoveLeft),
        (KeyCode::Char('L'), UIEvent::MoveRight),
        (KeyCode::Char('K'), UIEvent::MoveUp),
        (KeyCode::Char('J'), UIEvent::MoveDown),
        (KeyCode::Char('I'), UIEvent::InsertMode),
        (KeyCode::Char('E'), UIEvent::EditMode),
    ])
}
