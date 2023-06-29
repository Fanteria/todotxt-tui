use crate::error::ErrorType::ParseWidgetType;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use crate::error::ErrorToDo;

#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    Input,
    List,
    Done,
    Project,
    Context,
    Hashtag,
}

impl Display for WidgetType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use WidgetType::*;
        write!(
            f,
            "{}",
            match self {
                Input => "Input",
                List => "List",
                Done => "Done",
                Project => "Project",
                Context => "Context",
                Hashtag => "Hashtag",
            }
        )
    }
}

impl FromStr for WidgetType {
    type Err = ErrorToDo;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use WidgetType::*;
        match s.to_lowercase().as_str() {
            "input" => Ok(Input),
            "list" => Ok(List),
            "done" => Ok(Done),
            "project" => Ok(Project),
            "context" => Ok(Context),
            "hashtag" => Ok(Hashtag),
            _ => Err(ErrorToDo::new(ParseWidgetType, "Unknown widget type.")),
        }
    }
}
