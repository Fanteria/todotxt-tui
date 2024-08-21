use super::UIEvent;
use crate::{config::KeyCodeDef, ToDoError};
use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display, str::FromStr};

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

impl FromStr for EventEntry {
    type Err = crate::ToDoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(ToDoError::CannotParseEventEntry(
                "Too many separators :".to_string(),
            ));
        }
        let event = UIEvent::from_str(parts[1])?;
        let key = parts[0].to_lowercase();
        use KeyCode::*;
        Ok(match key.as_str() {
            "backspace" => EventEntry {
                key: Backspace,
                event,
            },
            "null" => EventEntry { key: Null, event },
            "esc" => EventEntry { key: Esc, event },
            "capslock" => EventEntry {
                key: CapsLock,
                event,
            },
            "scrolllock" => EventEntry {
                key: ScrollLock,
                event,
            },
            "numlock" => EventEntry {
                key: NumLock,
                event,
            },
            "printscreen" => EventEntry {
                key: PrintScreen,
                event,
            },
            "pause" => EventEntry { key: Pause, event },
            "menu" => EventEntry { key: Menu, event },
            "keypadbegin" => EventEntry {
                key: KeypadBegin,
                event,
            },
            "enter" => EventEntry { key: Enter, event },
            "left" => EventEntry { key: Left, event },
            "right" => EventEntry { key: Right, event },
            "up" => EventEntry { key: Up, event },
            "down" => EventEntry { key: Down, event },
            "home" => EventEntry { key: Home, event },
            "end" => EventEntry { key: End, event },
            "pageup" => EventEntry { key: PageUp, event },
            "pagedown" => EventEntry {
                key: PageDown,
                event,
            },
            "tab" => EventEntry { key: Tab, event },
            "backtab" => EventEntry {
                key: BackTab,
                event,
            },
            "delete" => EventEntry { key: Delete, event },
            "insert" => EventEntry { key: Insert, event },
            _ if key.len() == 1 => EventEntry {
                key: KeyCode::Char(
                    (key)
                        .parse::<char>()
                        .map_err(|e| ToDoError::CannotParseEventEntry(format!("{}", e)))?,
                ),
                event,
            },
            _ if key.starts_with('f') && key.len() > 1 => EventEntry {
                key: KeyCode::F(
                    (key[1..])
                        .parse()
                        .map_err(|e| ToDoError::CannotParseEventEntry(format!("{}", e)))?,
                ),
                event,
            },
            _ => return Err(ToDoError::CannotParseEventEntry("Unknown key".to_string())),
        })
    }
}

impl Display for EventEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.key {
            KeyCode::F(num) => write!(f, "F{}:{:?}", num, self.event),
            KeyCode::Char(c) => write!(f, "{}:{:?}", c, self.event),
            _ => write!(f, "{:?}:{:?}", self.key, self.event),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Result;

    use super::*;

    #[test]
    fn display() {
        assert_eq!(
            format!(
                "{}",
                EventEntry {
                    key: KeyCode::Char('f'),
                    event: UIEvent::Quit
                }
            ),
            "f:Quit"
        );

        assert_eq!(
            format!(
                "{}",
                EventEntry {
                    key: KeyCode::Backspace,
                    event: UIEvent::Load,
                }
            ),
            "Backspace:Load"
        );

        assert_eq!(
            format!(
                "{}",
                EventEntry {
                    key: KeyCode::F(5),
                    event: UIEvent::Save,
                }
            ),
            "F5:Save"
        );
    }

    #[test]
    fn from_str() -> Result<()> {
        assert_eq!(
            EventEntry {
                key: KeyCode::Char('a'),
                event: UIEvent::ListUp
            },
            EventEntry::from_str("a:ListUp")?
        );

        assert_eq!(
            EventEntry {
                key: KeyCode::Insert,
                event: UIEvent::Select
            },
            EventEntry::from_str("iNSert:select")?
        );

        assert_eq!(
            EventEntry {
                key: KeyCode::F(6),
                event: UIEvent::Remove
            },
            EventEntry::from_str("F6:rEmOvE")?
        );

        Ok(())
    }
}
