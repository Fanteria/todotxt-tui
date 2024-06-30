use super::UIEvent;
use crate::config::KeyCodeDef;
use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Struct representing an entry that maps a `KeyCode` to a `UIEvent`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventEntry {
    #[serde(with = "KeyCodeDef")]
    pub key: KeyCode,
    pub event: UIEvent,
}

impl PartialEq for EventEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl PartialEq<KeyCode> for EventEntry {
    fn eq(&self, other: &KeyCode) -> bool {
        self.key == *other
    }
}

impl PartialOrd<KeyCode> for EventEntry {
    fn partial_cmp(&self, other: &KeyCode) -> Option<Ordering> {
        self.key.partial_cmp(other)
    }
}

impl Eq for EventEntry {}

impl From<&(KeyCode, UIEvent)> for EventEntry {
    fn from(value: &(KeyCode, UIEvent)) -> Self {
        Self {
            key: value.0,
            event: value.1,
        }
    }
}
