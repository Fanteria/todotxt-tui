use super::TextStyle;
use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    str::FromStr,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct CustomCategoryStyle(HashMap<String, TextStyle>);

impl Deref for CustomCategoryStyle {
    type Target = HashMap<String, TextStyle>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CustomCategoryStyle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromStr for CustomCategoryStyle {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        fn parse(item: &str) -> Result<(String, TextStyle)> {
            let (key, value) = item.split_once('=').context(
                "Cannot parse custom category style: Key and value must be separated by =",
            )?;
            Ok((key.to_string(), TextStyle::from_str(value)?))
        }
        Ok(CustomCategoryStyle(
            s.split(',').map(parse).collect::<Result<_>>()?,
        ))
    }
}
