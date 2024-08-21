mod event_entry;

use crossterm::event::KeyCode;
use event_entry::EventEntry;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display, str::FromStr};

use crate::{Result, ToDoError};

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

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use UIEvent::*;
        Ok(match s.to_lowercase().as_str() {
            "quit" => Quit,
            "save" => Save,
            "load" => Load,
            "moveleft" => MoveLeft,
            "moveright" => MoveRight,
            "moveup" => MoveUp,
            "movedown" => MoveDown,
            "insertmode" => InsertMode,
            "editmode" => EditMode,

            "listdown" => ListDown,
            "listup" => ListUp,
            "listfirst" => ListFirst,
            "listlast" => ListLast,
            "swapupitem" => SwapUpItem,
            "swapdownitem" => SwapDownItem,
            "removeitem" => RemoveItem,
            "moveitem" => MoveItem,
            "select" => Select,
            "remove" => Remove,
            "none" => None,

            _ => {
                return Err(ToDoError::CannotParseUIEvent(format!(
                    "Unknown keyword {s}"
                )))
            }
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

    #[allow(unused_variables)]
    fn click(&mut self, column: usize, row: usize) {}
}

/// Struct for handling UI events based on key bindings.
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, Debug)]
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

    pub fn combine(&mut self, other: Self) {
        other.events.into_iter().for_each(|event| {
            match self.events.iter_mut().find(|e| e.key == event.key) {
                Some(e) => *e = event,
                None => self.events.push(event),
            }
        });
        self.events
            .sort_by(|left, right| left.key.partial_cmp(&right.key).unwrap_or(Ordering::Equal));
    }
}

impl FromStr for EventHandlerUI {
    type Err = crate::ToDoError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim();
        println!("---------------------------------------");

        let data = s
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .ok_or_else(|| ToDoError::CannotParseUIEvent("Value must be in []".to_string()))?
            .trim();

        Ok(EventHandlerUI {
            events: if data.is_empty() {
                Vec::new()
            } else {
                data.split(',')
                    .map(|s| {
                        println!("{s}");
                        EventEntry::from_str(s.trim())
                    })
                    .collect::<Result<_>>()?
            },
        })
    }
}

impl Display for EventHandlerUI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.events
                .iter()
                .map(|event| format!("{}", event))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_handler_ui_from_str() -> Result<()> {
        assert_eq!(
            EventHandlerUI::from_str("[]")?,
            EventHandlerUI { events: vec![] }
        );

        assert_eq!(
            EventHandlerUI::from_str("[f:none,capslock:moveitem]")?,
            EventHandlerUI {
                events: vec![
                    EventEntry {
                        key: KeyCode::Char('f'),
                        event: UIEvent::None,
                    },
                    EventEntry {
                        key: KeyCode::CapsLock,
                        event: UIEvent::MoveItem,
                    }
                ]
            }
        );

        Ok(())
    }

    #[test]
    fn event_handler_ui_display() {
        assert_eq!(format!("{}", EventHandlerUI { events: vec![] }), "[]");
        assert_eq!(
            format!(
                "{}",
                EventHandlerUI {
                    events: vec![
                        EventEntry {
                            key: KeyCode::Char('f'),
                            event: UIEvent::None,
                        },
                        EventEntry {
                            key: KeyCode::CapsLock,
                            event: UIEvent::MoveItem,
                        }
                    ]
                }
            ),
            "[f:None, CapsLock:MoveItem]"
        );
    }

    #[test]
    fn combine() {
        let mut base = EventHandlerUI::new(&[
            (KeyCode::Char('a'), UIEvent::ListDown),
            (KeyCode::Char('b'), UIEvent::ListUp),
        ]);
        let addition = EventHandlerUI::new(&[
            (KeyCode::Char('b'), UIEvent::MoveUp),
            (KeyCode::Char('c'), UIEvent::ListUp),
        ]);
        base.combine(addition);
        assert_eq!(
            base,
            EventHandlerUI::new(&[
                (KeyCode::Char('a'), UIEvent::ListDown),
                (KeyCode::Char('b'), UIEvent::MoveUp),
                (KeyCode::Char('c'), UIEvent::ListUp),
            ])
        );
    }
}
