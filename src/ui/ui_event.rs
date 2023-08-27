mod event_entry;

use std::cmp::Ordering;
use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};
use event_entry::EventEntry;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[cfg_attr(test, derive(Debug))]
pub enum UIEvent {
    Quit,

    ListDown, // Widget list
    ListUp,
    ListFirst,
    ListLast,
    SwapUpItem, // State list
    SwapDownItem,
    RemoveItem,
    MoveItem,
    Select, // State categories + State list
    // State preview
    None, // without bind
}

pub trait HandleEvent {
    fn get_event(&self, key: &KeyCode) -> UIEvent;
    fn handle_event(&mut self, event: UIEvent) -> bool;

    fn handle_key(&mut self, key: &KeyCode) -> bool {
        self.handle_event(self.get_event(key))
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct EventHandler {
    events: Vec<EventEntry>,
}

impl EventHandler {
    pub fn new(events: &[(KeyCode, UIEvent)]) -> Self {
        let mut events: Vec<_> = events.iter().map(|e| e.into()).collect();
        events.sort();
        Self { events }
    }

    #[allow(dead_code)]
    pub fn get_event(&self, key: &KeyCode) -> UIEvent {
        match self.events.binary_search_by(|a| Self::compare(&a.key, key)) {
            Ok(index) => self.events[index].event.clone(),
            Err(_) => UIEvent::None,
        }
    }

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
