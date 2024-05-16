mod event_entry;

use crossterm::event::KeyCode;
use event_entry::EventEntry;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, str::FromStr};

use crate::ToDoError;

/// Enum representing various UI events that can be triggered.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Copy, Debug)]
pub enum UIEvent {
    Quit, // Window
    Save,
    Load,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    InsertMode,
    EditMode,

    ListDown, // Widget list
    ListUp,
    ListFirst,
    ListLast,
    SwapUpItem, // State list
    SwapDownItem,
    RemoveItem,
    MoveItem,
    Select, // State categories + State list
    Remove, // State categories
    // State preview
    None, // without bind
}

impl FromStr for UIEvent {
    type Err = ToDoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use UIEvent::*;
        Ok(match s {
            "Quit" => Quit,
            "Save" => Save,
            "Load" => Load,
            "MoveLeft" => MoveLeft,
            "MoveRight" => MoveRight,
            "MoveUp" => MoveUp,
            "MoveDown" => MoveDown,
            "InsertMode" => InsertMode,
            "EditMode" => EditMode,

            "ListDown" => ListDown,
            "ListUp" => ListUp,
            "ListFirst" => ListFirst,
            "ListLast" => ListLast,
            "SwapUpItem" => SwapUpItem,
            "SwapDownItem" => SwapDownItem,
            "RemoveItem" => RemoveItem,
            "MoveItem" => MoveItem,
            "Select" => Select,
            "None" => None,

            _ => todo!(), // Error TODO
        })
    }
}

/// Trait for handling UI events.
pub trait HandleEvent {
    /// Get the UI event corresponding to a given key code.
    ///
    /// # Arguments
    ///
    /// * `key` - The key code to map to a UI event.
    ///
    /// # Returns
    ///
    /// The UI event corresponding to the key code.
    fn get_event(&self, key: &KeyCode) -> UIEvent;

    /// Handle a UI event.
    ///
    /// # Arguments
    ///
    /// * `event` - The UI event to handle.
    ///
    /// # Returns
    ///
    /// `true` if the event was successfully handled, `false` otherwise.
    fn handle_event(&mut self, event: UIEvent) -> bool;

    /// Handle a key press event.
    ///
    /// # Arguments
    ///
    /// * `key` - The key code representing the pressed key.
    ///
    /// # Returns
    ///
    /// `true` if the event was successfully handled, `false` otherwise.
    fn handle_key(&mut self, key: &KeyCode) -> bool {
        let event = self.get_event(key);
        log::trace!("EventHandler: Key '{:?}' cause event '{:?}'", key, event);
        self.handle_event(event)
    }
}

/// Struct for handling UI events based on key bindings.
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct EventHandlerUI {
    events: Vec<EventEntry>,
}

impl EventHandlerUI {
    /// Create a new `EventHandler` with the provided key bindings.
    ///
    /// # Arguments
    ///
    /// * `events` - A slice of key bindings as tuples of `(KeyCode, UIEvent)`.
    ///
    /// # Returns
    ///
    /// A new `EventHandler` instance.
    pub fn new(events: &[(KeyCode, UIEvent)]) -> Self {
        let mut events: Vec<EventEntry> = events.iter().map(|e| e.into()).collect();
        events.sort_by(|left, right| left.key.partial_cmp(&right.key).unwrap_or(Ordering::Equal));
        Self { events }
    }

    /// Get the UI event corresponding to a given key code.
    ///
    /// # Arguments
    ///
    /// * `key` - The key code to map to a UI event.
    ///
    /// # Returns
    ///
    /// The UI event corresponding to the key code.
    pub fn get_event(&self, key: &KeyCode) -> UIEvent {
        match self.events.binary_search_by(|a| Self::compare(&a.key, key)) {
            Ok(index) => self.events[index].event,
            Err(_) => UIEvent::None,
        }
    }

    /// Compare two key codes for ordering purposes.
    ///
    /// # Arguments
    ///
    /// * `a` - The first key code to compare.
    /// * `b` - The second key code to compare.
    ///
    /// # Returns
    ///
    /// The ordering of the key codes.
    fn compare(a: &KeyCode, b: &KeyCode) -> Ordering {
        if a < b {
            Ordering::Less
        } else if a > b {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
