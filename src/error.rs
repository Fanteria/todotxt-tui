use std::{
    error::Error,
    fmt::{self, Display},
    num::{IntErrorKind, ParseIntError},
};

/// Define a custom result type for ToDo related operations.
pub type ToDoRes<T> = Result<T, ToDoError>;

// Define error messages.
const WIDGET_DOES_NOT_EXIST: &str = "Selected widgent is not in layout";
const PARSE_WIDGET_TYPE: &str = "Unknown widget type.";
const PARSE_NOT_START: &str = "There must be almost one container.";
const PARSE_NOT_END: &str = "All containers must be closed";
const ACTIVE_IS_NOT_WIDGET: &str = "Invalid state, active container is not widget.";

/// Enum representing ToDo-related errors.
#[derive(Debug)]
pub enum ToDoError {
    WidgetDoesNotExist,
    ParseValue(IntErrorKind),
    ParseUnknownValue,
    ParseWidgetType,
    ParseNotStart,
    ParseNotEnd,
    ParseInvalidDirection(String),
    ActiveIsNotWidget,
}

/// Implement the Error trait for ToDoError.
impl Error for ToDoError {}

/// Implement the Display trait for ToDoError to format error messages.
impl Display for ToDoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut msg = match self {
            Self::WidgetDoesNotExist => String::from(WIDGET_DOES_NOT_EXIST),
            Self::ParseValue(e) => format!("{:?}", e),
            Self::ParseWidgetType => String::from(PARSE_WIDGET_TYPE),
            Self::ParseNotStart => String::from(PARSE_NOT_START),
            Self::ParseNotEnd => String::from(PARSE_NOT_END),
            Self::ParseInvalidDirection(direction) => {
                format!("Direction \"{}\" is invalid", direction)
            }
            Self::ActiveIsNotWidget => String::from(ACTIVE_IS_NOT_WIDGET),
            _ => String::new(),
        };
        if !msg.is_empty() {
            msg = format!(": {}", msg);
        }
        write!(f, "Error: \"{}\"{}", self.name(), msg)
    }
}

impl ToDoError {
    /// Get a error name.
    pub fn name(&self) -> String {
        match self {
            Self::WidgetDoesNotExist => String::from("widget does not exists"),
            Self::ParseValue(_) => String::from("parse value error"),
            Self::ParseUnknownValue => {
                String::from("Value can constraint only unsigned integer and %.")
            }
            Self::ParseWidgetType => String::from("parse widget type"),
            Self::ParseNotStart => String::from("parse not start"),
            Self::ParseNotEnd => String::from("parse not end"),
            Self::ParseInvalidDirection(_) => String::from("parse invalid direction"),
            Self::ActiveIsNotWidget => String::from("active is not widget"),
        }
    }
}

/// Conversion from ParseIntError to ToDoError.
impl From<ParseIntError> for ToDoError {
    fn from(e: ParseIntError) -> Self {
        Self::ParseValue(e.kind().clone())
    }
}
