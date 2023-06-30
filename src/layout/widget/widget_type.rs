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
                Project => "Projects",
                Context => "Contexts",
                Hashtag => "Hashtags",
            }
        )
    }
}

impl FromStr for WidgetType {
    type Err = ErrorToDo;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use WidgetType::*;
        println!("{:#?}", s);
        match s.to_lowercase().as_str() {
            "input" => Ok(Input),
            "list" => Ok(List),
            "done" => Ok(Done),
            "projects" => Ok(Project),
            "contexts" => Ok(Context),
            "hashtags" => Ok(Hashtag),
            _ => Err(ErrorToDo::new(ParseWidgetType, "Unknown widget type.")),
        }
    }
}
