use crate::ToDoError;
use serde::{de, Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    str::FromStr,
};
use tui::style::Color as tuiColor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub tuiColor);

impl Color {
    pub fn black() -> Self {
        Self(tuiColor::Black)
    }
    pub fn red() -> Self {
        Self(tuiColor::Red)
    }
    pub fn green() -> Self {
        Self(tuiColor::Green)
    }
    pub fn yellow() -> Self {
        Self(tuiColor::Yellow)
    }
    pub fn blue() -> Self {
        Self(tuiColor::Blue)
    }
    pub fn magenta() -> Self {
        Self(tuiColor::Magenta)
    }
    pub fn cyan() -> Self {
        Self(tuiColor::Cyan)
    }
    pub fn gray() -> Self {
        Self(tuiColor::Gray)
    }
    pub fn darkgray() -> Self {
        Self(tuiColor::DarkGray)
    }
    pub fn lightred() -> Self {
        Self(tuiColor::LightRed)
    }
    pub fn lightgreen() -> Self {
        Self(tuiColor::LightGreen)
    }
    pub fn lightyellow() -> Self {
        Self(tuiColor::LightYellow)
    }
    pub fn lightblue() -> Self {
        Self(tuiColor::LightBlue)
    }
    pub fn lightmagenta() -> Self {
        Self(tuiColor::LightMagenta)
    }
    pub fn lightcyan() -> Self {
        Self(tuiColor::LightCyan)
    }
    pub fn white() -> Self {
        Self(tuiColor::White)
    }
}

impl Deref for Color {
    type Target = tui::style::Color;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Color {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            tuiColor::Rgb(r, g, b) => write!(f, "#{r:02x}{g:02x}{b:02x}"),
            tuiColor::Indexed(i) => write!(f, "{i}"),
            _ => write!(f, "{:?}", self.0),
        }
    }
}

impl From<tuiColor> for Color {
    fn from(value: tuiColor) -> Self {
        Self(value)
    }
}

impl FromStr for Color {
    type Err = ToDoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_hex = |hex: &str| -> Result<u8, Self::Err> {
            u8::from_str_radix(hex, 16)
                .map_err(|_e| ToDoError::ColorSerializationFailed(s.to_string()))
        };

        if let Ok(index) = s.parse::<u8>() {
            return Ok(Self(tuiColor::Indexed(index)));
        }
        let lower = s.to_lowercase();
        use tuiColor::*;
        Ok(Self(match lower.as_str() {
            "black" => Black,
            "red" => Red,
            "green" => Green,
            "yellow" => Yellow,
            "blue" => Blue,
            "magenta" => Magenta,
            "cyan" => Cyan,
            "gray" => Gray,
            "darkgray" => DarkGray,
            "lightred" => LightRed,
            "lightgreen" => LightGreen,
            "lightyellow" => LightYellow,
            "lightblue" => LightBlue,
            "lightmagenta" => LightMagenta,
            "lightcyan" => LightCyan,
            "white" => White,
            _ if lower.starts_with('#') && lower.len() == 7 => Rgb(
                parse_hex(&lower[1..3])?,
                parse_hex(&lower[3..5])?,
                parse_hex(&lower[5..7])?,
            ),
            _ => return Err(ToDoError::ColorSerializationFailed(s.to_string())),
        }))
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    #[test]
    fn from_str() -> Result<()> {
        assert_eq!(Color(tuiColor::Red), Color::from_str("red")?);
        assert_eq!(Color(tuiColor::Yellow), Color::from_str("YeLlOW")?);
        assert_eq!(Color(tuiColor::Rgb(3, 4, 5)), Color::from_str("#030405")?);
        Ok(())
    }

    #[test]
    fn to_string() {
        assert_eq!(Color(tuiColor::Green).to_string(), "Green");
        assert_eq!(Color(tuiColor::Black).to_string(), "Black");
        assert_eq!(Color(tuiColor::Rgb(6, 7, 8)).to_string(), "#060708");
    }
}
