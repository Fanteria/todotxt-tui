use crossterm::event::KeyCode;
use serde::{de, Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, ops::Deref, str::FromStr};

use crate::{Result, ToDoError};

#[derive(Clone, PartialEq, Eq, Copy, Debug, Hash)]
pub struct KeyCodeA(KeyCode);

impl Deref for KeyCodeA {
    type Target = KeyCode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for KeyCodeA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            KeyCode::F(num) => write!(f, "F{}", num),
            KeyCode::Char(c) => write!(f, "{}", c),
            _ => write!(f, "{:?}", self.0),
        }
    }
}

impl Serialize for KeyCodeA {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for KeyCodeA {
    type Err = crate::ToDoError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "backspace" => Self(KeyCode::Backspace),
            "null" => Self(KeyCode::Null),
            "esc" => Self(KeyCode::Esc),
            "capslock" => Self(KeyCode::CapsLock),
            "scrolllock" => Self(KeyCode::ScrollLock),
            "numlock" => Self(KeyCode::NumLock),
            "printscreen" => Self(KeyCode::PrintScreen),
            "pause" => Self(KeyCode::Pause),
            "menu" => Self(KeyCode::Menu),
            "keypadbegin" => Self(KeyCode::KeypadBegin),
            "enter" => Self(KeyCode::Enter),
            "left" => Self(KeyCode::Left),
            "right" => Self(KeyCode::Right),
            "up" => Self(KeyCode::Up),
            "down" => Self(KeyCode::Down),
            "home" => Self(KeyCode::Home),
            "end" => Self(KeyCode::End),
            "pageup" => Self(KeyCode::PageUp),
            "pagedown" => Self(KeyCode::PageDown),
            "tab" => Self(KeyCode::Tab),
            "backtab" => Self(KeyCode::BackTab),
            "delete" => Self(KeyCode::Delete),
            "insert" => Self(KeyCode::Insert),
            _ if s.len() == 1 => {
                Self(KeyCode::Char((s).parse::<char>().map_err(|e| {
                    ToDoError::CannotParseEventEntry(format!("{}", e))
                })?))
            }
            _ if s.starts_with('F') && s.len() > 1 => {
                Self(KeyCode::F((s[1..]).parse().map_err(|e| {
                    ToDoError::CannotParseEventEntry(format!("{}", e))
                })?))
            }
            _ => return Err(ToDoError::CannotParseEventEntry("Unknown key".to_string())),
        })
    }
}

impl<'de> Deserialize<'de> for KeyCodeA {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}

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
    SearchMode,

    CleanSearch,
    NextSearch,
    PrevSearch,
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
            "searchmode" => SearchMode,

            "cleansearch" => CleanSearch,
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

/// Struct for handling UI events based on key bindings.
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, Debug)]
pub struct EventHandlerUI(HashMap<KeyCodeA, UIEvent>);

impl EventHandlerUI {
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
        *self.0.get(&KeyCodeA(*key)).unwrap_or(&UIEvent::None)
    }

    /// Combines the elements of another `Vec` into the current instance,
    /// extending the current vector with the elements from the provided vector.
    pub fn combine(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}

impl FromStr for EventHandlerUI {
    type Err = crate::ToDoError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim();

        let data = s
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .ok_or_else(|| ToDoError::CannotParseUIEvent("Value must be in []".to_string()))?
            .trim();

        Ok(EventHandlerUI(if data.is_empty() {
            HashMap::new()
        } else {
            data.split(',')
                .map(|s| {
                    let (key, event) = s.split_once(':').ok_or_else(|| {
                        ToDoError::CannotParseEventEntry("Missing separator :".to_string())
                    })?;
                    Ok((KeyCodeA::from_str(key)?, UIEvent::from_str(event)?))
                })
                .collect::<Result<HashMap<_, _>>>()?
        }))
    }
}

impl Display for EventHandlerUI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut maps = self
            .0
            .iter()
            .map(|(key, event)| match key.0 {
                KeyCode::F(num) => format!("F{}:{:?}", num, event),
                KeyCode::Char(c) => format!("{}:{:?}", c, event),
                _ => format!("{:?}:{:?}", key.0, event),
            })
            .collect::<Vec<_>>();
        maps.sort();
        write!(f, "[{}]", maps.join(", "))
    }
}

impl<const N: usize> From<[(KeyCode, UIEvent); N]> for EventHandlerUI {
    fn from(value: [(KeyCode, UIEvent); N]) -> Self {
        EventHandlerUI(
            value
                .into_iter()
                .map(|(key, event)| (KeyCodeA(key), event))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization() -> Result<()> {
        let event_handler = EventHandlerUI::from([
            (KeyCode::Char('f'), UIEvent::None),
            (KeyCode::CapsLock, UIEvent::MoveItem),
        ]);
        let mut events = toml::to_string(&event_handler)?
            .lines()
            .map(|l| l.to_string())
            .collect::<Vec<_>>();
        events.sort();
        assert_eq!(events, vec!["CapsLock = \"MoveItem\"", "f = \"None\""]);

        Ok(())
    }

    #[test]
    fn deserialize() -> Result<()> {
        assert_eq!(
            toml::from_str::<EventHandlerUI>("CapsLock = \"MoveItem\"\nf = \"None\"")?,
            EventHandlerUI::from([
                (KeyCode::Char('f'), UIEvent::None),
                (KeyCode::CapsLock, UIEvent::MoveItem),
            ]),
        );

        Ok(())
    }

    #[test]
    fn event_handler_ui_from_str() -> Result<()> {
        assert_eq!(
            EventHandlerUI::from_str("[]")?,
            EventHandlerUI(HashMap::new()),
        );
        assert_eq!(
            EventHandlerUI::from_str("[f:none,capslock:moveitem]")?,
            EventHandlerUI::from([
                (KeyCode::Char('f'), UIEvent::None),
                (KeyCode::CapsLock, UIEvent::MoveItem),
            ])
        );

        Ok(())
    }

    #[test]
    fn event_handler_ui_display() {
        assert_eq!(format!("{}", EventHandlerUI(HashMap::new()),), "[]");
        assert_eq!(
            format!(
                "{}",
                EventHandlerUI::from([
                    (KeyCode::Char('f'), UIEvent::None),
                    (KeyCode::CapsLock, UIEvent::MoveItem),
                ])
            ),
            "[CapsLock:MoveItem, f:None]"
        );
    }

    #[test]
    fn combine() {
        let mut base = EventHandlerUI::from([
            (KeyCode::Char('a'), UIEvent::ListDown),
            (KeyCode::Char('b'), UIEvent::ListUp),
        ]);
        let addition = EventHandlerUI::from([
            (KeyCode::Char('b'), UIEvent::MoveUp),
            (KeyCode::Char('c'), UIEvent::ListUp),
        ]);
        base.combine(addition);
        assert_eq!(
            base,
            EventHandlerUI::from([
                (KeyCode::Char('a'), UIEvent::ListDown),
                (KeyCode::Char('b'), UIEvent::MoveUp),
                (KeyCode::Char('c'), UIEvent::ListUp),
            ])
        );
    }

    #[test]
    fn event_entry_display() {
        assert_eq!(
            &EventHandlerUI::from([(KeyCode::Char('f'), UIEvent::Quit)]).to_string(),
            "[f:Quit]"
        );

        assert_eq!(
            &EventHandlerUI::from([(KeyCode::Backspace, UIEvent::Load)]).to_string(),
            "[Backspace:Load]"
        );

        assert_eq!(
            &EventHandlerUI::from([(KeyCode::F(5), UIEvent::Save)]).to_string(),
            "[F5:Save]"
        );
    }

    #[test]
    fn event_entry_from_str() -> Result<()> {
        assert_eq!(
            EventHandlerUI::from([(KeyCode::Char('a'), UIEvent::ListUp)]),
            EventHandlerUI::from_str("[a:ListUp]")?
        );

        assert_eq!(
            EventHandlerUI::from([(KeyCode::Insert, UIEvent::Select)]),
            EventHandlerUI::from_str("[iNSert:select]")?
        );

        assert_eq!(
            EventHandlerUI::from([(KeyCode::F(6), UIEvent::Remove)]),
            EventHandlerUI::from_str("[F6:rEmOvE]")?
        );

        Ok(())
    }
}
