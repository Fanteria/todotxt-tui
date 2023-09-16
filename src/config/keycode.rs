use crossterm::event::ModifierKeyCode;
use crossterm::event::MediaKeyCode;
use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};

/// Serialization and deserialization support for the TUI keycode type.
///
/// This enum is used to serialize and deserialize TUI `KeyCode` objects.
#[derive(Serialize, Deserialize)]
#[serde(remote = "KeyCode")]
pub enum KeyCodeDef {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Null,
    Esc,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
    KeypadBegin,
    #[serde(with = "MediaKeyCodeDef")]
    Media(MediaKeyCode),
    #[serde(with = "ModifierKeyCodeDef")]
    Modifier(ModifierKeyCode),
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "MediaKeyCode")]
pub enum MediaKeyCodeDef {
    Play,
    Pause,
    PlayPause,
    Reverse,
    Stop,
    FastForward,
    Rewind,
    TrackNext,
    TrackPrevious,
    Record,
    LowerVolume,
    RaiseVolume,
    MuteVolume,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "ModifierKeyCode")]
pub enum ModifierKeyCodeDef {
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    LeftHyper,
    LeftMeta,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    RightHyper,
    RightMeta,
    IsoLevel3Shift,
    IsoLevel5Shift,
}
