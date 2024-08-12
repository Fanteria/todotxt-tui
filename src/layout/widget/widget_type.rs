use crate::{
    todo::{ToDoCategory, ToDoData},
    ToDoError,
};
use clap::ValueEnum;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// An enumeration representing different types of widgets used in the application.
/// Widgets are I components with specific functionalities, such as task lists, project lists, and previews.
#[derive(Default, PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize, ValueEnum)]
pub enum WidgetType {
    #[default]
    List,
    Done,
    Project,
    Context,
    Hashtag,
    Preview,
}

impl fmt::Display for WidgetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{self:?}").to_lowercase())
    }
}

impl From<ToDoCategory> for WidgetType {
    fn from(value: ToDoCategory) -> Self {
        use ToDoCategory::*;
        match value {
            Projects => WidgetType::Project,
            Contexts => WidgetType::Context,
            Hashtags => WidgetType::Hashtag,
        }
    }
}

impl From<ToDoData> for WidgetType {
    fn from(value: ToDoData) -> Self {
        use ToDoData::*;
        match value {
            Pending => WidgetType::List,
            Done => WidgetType::Done,
        }
    }
}

impl FromStr for WidgetType {
    type Err = ToDoError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use WidgetType::*;
        Ok(match s.to_lowercase().as_str() {
            "list" => List,
            "done" => Done,
            "projects" => Project,
            "contexts" => Context,
            "hashtags" => Hashtag,
            "preview" => Preview,
            _ => return Err(ToDoError::ParseWidgetType(s.to_string())),
        })
    }
}
