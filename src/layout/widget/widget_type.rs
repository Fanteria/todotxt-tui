use crate::{
    error::ErrorType::ParseWidgetType,
    todo::{ToDoCategory, ToDoData},
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::error::ErrorToDo;

#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    List,
    Done,
    Project,
    Context,
    Hashtag,
    Preview,
}

impl ToString for WidgetType {
    fn to_string(&self) -> String {
        use WidgetType::*;
        match self {
            List => String::from("List"),
            Done => String::from("Done"),
            Project => String::from("Projects"),
            Context => String::from("Contexts"),
            Hashtag => String::from("Hashtags"),
            Preview => String::from("Preview"),
        }
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
    type Err = ErrorToDo;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use WidgetType::*;
        match s.to_lowercase().as_str() {
            "list" => Ok(List),
            "done" => Ok(Done),
            "projects" => Ok(Project),
            "contexts" => Ok(Context),
            "hashtags" => Ok(Hashtag),
            "preview" => Ok(Preview),
            _ => Err(ErrorToDo::new(ParseWidgetType, "Unknown widget type.")),
        }
    }
}
