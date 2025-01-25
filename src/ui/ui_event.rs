use crate::{Result, ToDoError};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{de, Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, str::FromStr};

#[derive(Clone, PartialEq, Eq, Copy, Debug, Hash)]
pub struct KeyShortcut {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyShortcut {
    pub fn new(key: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }
}

impl From<KeyCode> for KeyShortcut {
    fn from(value: KeyCode) -> Self {
        Self::new(value, KeyModifiers::NONE)
    }
}

impl From<&KeyEvent> for KeyShortcut {
    fn from(value: &KeyEvent) -> Self {
        Self::new(
            match value.code {
                KeyCode::Char(c) => KeyCode::Char(c.to_ascii_lowercase()),
                _ => value.code,
            },
            value.modifiers,
        )
    }
}

impl Display for KeyShortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.modifiers != KeyModifiers::NONE {
            write!(f, "{}+", self.modifiers)?;
        }
        match self.key {
            KeyCode::F(num) => write!(f, "F{}", num),
            KeyCode::Char(c) => write!(f, "{}", c),
            _ => write!(f, "{:?}", self.key),
        }
    }
}

impl Serialize for KeyShortcut {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for KeyShortcut {
    type Err = crate::ToDoError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut splitted = s.split('+').rev();
        let s = splitted.next().unwrap();
        let modifiers = splitted
            .map(|s| match s.to_lowercase().as_str() {
                "s" | "shift" => Ok(KeyModifiers::SHIFT),
                "c" | "ctrl" => Ok(KeyModifiers::CONTROL),
                "a" | "alt" => Ok(KeyModifiers::ALT),
                _ => Err(ToDoError::CannotParseEventEntry(
                    "Unknown modifier".to_string(),
                )),
            })
            .try_fold(KeyModifiers::NONE, |acc, modifier| {
                Ok::<_, ToDoError>(acc | modifier?)
            })?;
        Ok(match s.to_lowercase().as_str() {
            "backspace" => Self::new(KeyCode::Backspace, modifiers),
            "null" => Self::new(KeyCode::Null, modifiers),
            "esc" | "escape" => Self::new(KeyCode::Esc, modifiers),
            "capslock" => Self::new(KeyCode::CapsLock, modifiers),
            "scrolllock" => Self::new(KeyCode::ScrollLock, modifiers),
            "numlock" => Self::new(KeyCode::NumLock, modifiers),
            "printscreen" => Self::new(KeyCode::PrintScreen, modifiers),
            "pause" => Self::new(KeyCode::Pause, modifiers),
            "menu" => Self::new(KeyCode::Menu, modifiers),
            "keypadbegin" => Self::new(KeyCode::KeypadBegin, modifiers),
            "enter" => Self::new(KeyCode::Enter, modifiers),
            "left" => Self::new(KeyCode::Left, modifiers),
            "right" => Self::new(KeyCode::Right, modifiers),
            "up" => Self::new(KeyCode::Up, modifiers),
            "down" => Self::new(KeyCode::Down, modifiers),
            "home" => Self::new(KeyCode::Home, modifiers),
            "end" => Self::new(KeyCode::End, modifiers),
            "pageup" => Self::new(KeyCode::PageUp, modifiers),
            "pagedown" => Self::new(KeyCode::PageDown, modifiers),
            "tab" => Self::new(KeyCode::Tab, modifiers),
            "backtab" => Self::new(KeyCode::BackTab, modifiers),
            "delete" => Self::new(KeyCode::Delete, modifiers),
            "insert" => Self::new(KeyCode::Insert, modifiers),
            "plus" => Self::new(KeyCode::Char('+'), modifiers),
            "comma" => Self::new(KeyCode::Char(','), modifiers),
            "doubledot" => Self::new(KeyCode::Char(':'), modifiers),
            _ if s.len() == 1 => Self::new(
                KeyCode::Char(
                    (s).parse::<char>()
                        .map_err(|e| ToDoError::CannotParseEventEntry(format!("{}", e)))?
                        .to_ascii_lowercase(),
                ),
                modifiers,
            ),
            _ if s.starts_with('F') && s.len() > 1 => Self::new(
                KeyCode::F(
                    (s[1..])
                        .parse()
                        .map_err(|e| ToDoError::CannotParseEventEntry(format!("{}", e)))?,
                ),
                modifiers,
            ),
            _ => return Err(ToDoError::CannotParseEventEntry("Unknown key".to_string())),
        })
    }
}

impl<'de> Deserialize<'de> for KeyShortcut {
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
pub struct EventHandlerUI(HashMap<KeyShortcut, UIEvent>);

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
    pub fn get_event(&self, key: &KeyEvent) -> UIEvent {
        *self.0.get(&key.into()).unwrap_or(&UIEvent::None)
    }

    /// Combines the elements of another `Vec` into the current instance,
    /// extending the current vector with the elements from the provided vector.
    pub fn combine(&mut self, other: Self) {
        self.0.extend(other.0);
    }

    /// Check if keymaps are empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Number of keymaps.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Iterator of the key shortcuts.
    pub fn keys(&self) -> impl Iterator<Item = &KeyShortcut> {
        self.0.keys()
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
                    Ok((KeyShortcut::from_str(key)?, UIEvent::from_str(event)?))
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
            .map(|(key, event)| format!("{}:{:?}", key, event))
            .collect::<Vec<_>>();
        maps.sort();
        write!(f, "[{}]", maps.join(", "))
    }
}

impl<const N: usize> From<[(KeyShortcut, UIEvent); N]> for EventHandlerUI {
    fn from(value: [(KeyShortcut, UIEvent); N]) -> Self {
        EventHandlerUI(value.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization() -> Result<()> {
        let event_handler = EventHandlerUI::from([
            (KeyShortcut::from(KeyCode::Char('f')), UIEvent::None),
            (KeyShortcut::from(KeyCode::CapsLock), UIEvent::MoveItem),
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
                (KeyShortcut::from(KeyCode::Char('f')), UIEvent::None),
                (KeyShortcut::from(KeyCode::CapsLock), UIEvent::MoveItem),
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
                (KeyShortcut::from(KeyCode::Char('f')), UIEvent::None),
                (KeyShortcut::from(KeyCode::CapsLock), UIEvent::MoveItem),
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
                    (KeyShortcut::from(KeyCode::Char('f')), UIEvent::None),
                    (KeyShortcut::from(KeyCode::CapsLock), UIEvent::MoveItem),
                ])
            ),
            "[CapsLock:MoveItem, f:None]"
        );
    }

    #[test]
    fn combine() {
        let mut base = EventHandlerUI::from([
            (KeyShortcut::from(KeyCode::Char('a')), UIEvent::ListDown),
            (KeyShortcut::from(KeyCode::Char('b')), UIEvent::ListUp),
        ]);
        let addition = EventHandlerUI::from([
            (KeyShortcut::from(KeyCode::Char('b')), UIEvent::MoveUp),
            (KeyShortcut::from(KeyCode::Char('c')), UIEvent::ListUp),
        ]);
        base.combine(addition);
        assert_eq!(
            base,
            EventHandlerUI::from([
                (KeyShortcut::from(KeyCode::Char('a')), UIEvent::ListDown),
                (KeyShortcut::from(KeyCode::Char('b')), UIEvent::MoveUp),
                (KeyShortcut::from(KeyCode::Char('c')), UIEvent::ListUp),
            ])
        );
    }

    #[test]
    fn event_entry_display() {
        assert_eq!(
            &EventHandlerUI::from([(KeyShortcut::from(KeyCode::Char('f')), UIEvent::Quit)])
                .to_string(),
            "[f:Quit]"
        );

        assert_eq!(
            &EventHandlerUI::from([(KeyShortcut::from(KeyCode::Backspace), UIEvent::Load)])
                .to_string(),
            "[Backspace:Load]"
        );

        assert_eq!(
            &EventHandlerUI::from([(KeyShortcut::from(KeyCode::F(5)), UIEvent::Save)]).to_string(),
            "[F5:Save]"
        );

        assert_eq!(
            &EventHandlerUI::from([(
                KeyShortcut::new(KeyCode::F(5), KeyModifiers::ALT),
                UIEvent::Save
            )])
            .to_string(),
            "[Alt+F5:Save]"
        );
    }

    #[test]
    fn event_entry_from_str() -> Result<()> {
        assert_eq!(
            EventHandlerUI::from([(KeyShortcut::from(KeyCode::Char('a')), UIEvent::ListUp)]),
            EventHandlerUI::from_str("[a:ListUp]")?
        );

        assert_eq!(
            EventHandlerUI::from([(KeyShortcut::from(KeyCode::Insert), UIEvent::Select)]),
            EventHandlerUI::from_str("[iNSert:select]")?
        );

        assert_eq!(
            EventHandlerUI::from([(KeyShortcut::from(KeyCode::F(6)), UIEvent::Remove)]),
            EventHandlerUI::from_str("[F6:rEmOvE]")?
        );

        assert_eq!(
            EventHandlerUI::from([(
                KeyShortcut::new(KeyCode::Char('b'), KeyModifiers::SHIFT),
                UIEvent::ListDown
            )]),
            EventHandlerUI::from_str("[S+b:ListDown]")?
        );

        assert_eq!(
            EventHandlerUI::from([(
                KeyShortcut::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
                UIEvent::ListLast
            )]),
            EventHandlerUI::from_str("[Ctrl+b:ListLast]")?
        );

        assert_eq!(
            EventHandlerUI::from([(
                KeyShortcut::new(KeyCode::Char('b'), KeyModifiers::ALT),
                UIEvent::ListLast
            )]),
            EventHandlerUI::from_str("[A+B:ListLast]")?
        );

        Ok(())
    }
}
