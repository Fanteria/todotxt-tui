use crossterm::event::KeyCode;
use std::{path::PathBuf, time::Duration};

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::{layout::widget::widget_type::WidgetType, ui::{EventHandlerUI, UIEvent}};

#[derive(Serialize, Deserialize, Parser, Debug, PartialEq, Eq, Clone)]
pub struct UiConfig {
    /// Widget that will be active after start of the application.
    #[arg(long, default_value_t = default_init_widget(), value_name = "WIDGET_TYPE")]
    #[serde(default = "default_init_widget")]
    pub init_widget: WidgetType,

    /// Title of window with opened todotxt-tui
    #[arg(short = 'T', long, default_value_t = default_window_title(), value_name = "STRING")]
    #[serde(default = "default_window_title")]
    pub window_title: String,

    #[clap(skip)]
    #[serde(default = "default_window_keybinds")]
    pub window_keybinds: EventHandlerUI,

    /// List refresh rate (in seconds).
    #[arg(short = 'L', long, default_value = default_list_refresh_rate().as_secs().to_string(), value_parser = super::parsers::parse_duration, value_name = "DURATION")]
    #[serde(default = "default_list_refresh_rate")]
    pub list_refresh_rate: Duration,

    #[arg(long)]
    #[serde(default)]
    pub save_state_path: Option<PathBuf>, // TODO at now unused

    /// Layout configuration.
    #[arg(long, default_value_t = default_layout(), hide_default_value = true)]
    #[serde(default = "default_layout")]
    pub layout: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            init_widget: default_init_widget(),
            window_title: default_window_title(),
            window_keybinds: default_window_keybinds(),
            list_refresh_rate: default_list_refresh_rate(),
            save_state_path: None,
            layout: default_layout(),
        }
    }
}

fn default_init_widget() -> WidgetType {
    WidgetType::List
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

fn default_layout() -> String {
    String::from(concat!(
        "[",
        "  Direction: Horizontal,",
        "  Size: 50%,",
        "  [",
        "    List: 80%, Preview: 20%,",
        "  ],",
        "  [",
        "    Direction: Vertical,",
        "    Done: 60%,",
        "    [", 
        "      Contexts: 50%,",
        "      Projects: 50%,",
        "    ],",
        "  ],",
        "]",
    ))
}
