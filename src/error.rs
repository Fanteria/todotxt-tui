use std::{
    error::Error,
    fmt::{self, Display},
    num::{IntErrorKind, ParseIntError},
    path::PathBuf,
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
#[derive(Debug, PartialEq)]
pub enum ToDoError {
    WidgetDoesNotExist,
    ParseValue(IntErrorKind),
    ParseUnknownValue,
    ParseWidgetType,
    ParseNotStart,
    ParseNotEnd,
    ParseInvalidDirection(String),
    ParseTextStyle(String),
    ParseTextModifier(String),
    ParseBlockEscapeOnEnd(String),
    ParseBlockNotClosed(String),
    ParseVariableNotClosed(String),
    EmptyVariableName(String),
    ActiveIsNotWidget,
    IOoperationFailed(PathBuf, std::io::ErrorKind),
}

/// Implement the Error trait for ToDoError.
impl Error for ToDoError {}

/// Implement the Display trait for ToDoError to format error messages.
impl Display for ToDoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ToDoError::*;
        let mut msg = match self {
            WidgetDoesNotExist => String::from(WIDGET_DOES_NOT_EXIST),
            ParseValue(e) => format!("{:?}", e),
            ParseWidgetType => String::from(PARSE_WIDGET_TYPE),
            ParseNotStart => String::from(PARSE_NOT_START),
            ParseNotEnd => String::from(PARSE_NOT_END),
            ParseInvalidDirection(direction) => format!("Direction \"{}\" is invalid", direction),
            ParseTextStyle(style) => format!("Style \"{}\" is invalid", style),
            ParseTextModifier(modifier) => format!("Modifier \"{}\" is invalid", modifier),
            ParseBlockEscapeOnEnd(block) => format!("Block \"{}\" have escape on the end", block),
            ParseBlockNotClosed(block) => format!("Block \"{}\" is not closed", block),
            ParseVariableNotClosed(variable) => format!("Variable \"{}\" is not closed", variable),
            EmptyVariableName(block) => {
                format!("Block \"{}\" constraint empty variable name", block)
            }
            ActiveIsNotWidget => String::from(ACTIVE_IS_NOT_WIDGET),
            IOoperationFailed(path, kind) => {
                let x = match path.to_str() {
                    Some(name) => String::from(name),
                    None => String::from(""),
                };
                format!("{} {}", x, kind)
            }
            _ => String::new(),
        };
        if !msg.is_empty() {
            msg = format!(": {}", msg);
        }
        write!(f, "Error: \"{}\" {}", self.name(), msg)
    }
}

impl ToDoError {
    /// Get a error name.
    pub fn name(&self) -> &'static str {
        use ToDoError::*;
        match self {
            WidgetDoesNotExist => "widget does not exists",
            ParseValue(_) => "parse value error",
            ParseUnknownValue => "Value can constraint only unsigned integer and %.",
            ParseWidgetType => "parse widget type",
            ParseNotStart => "parse not start",
            ParseNotEnd => "parse not end",
            ParseInvalidDirection(_) => "parse invalid direction",
            ParseTextStyle(_) => "parse text style",
            ParseTextModifier(_) => "parse text modifier",
            ParseBlockEscapeOnEnd(_) => "parse block have escape on the end",
            ParseBlockNotClosed(_) => "parse block not closed",
            ParseVariableNotClosed(_) => "parse variable not closed",
            EmptyVariableName(_) => "empty variable name",
            ActiveIsNotWidget => "active is not widget",
            IOoperationFailed(_, _) => "IO operation failed",
        }
    }
}

/// Conversion from ParseIntError to ToDoError.
impl From<ParseIntError> for ToDoError {
    fn from(e: ParseIntError) -> Self {
        Self::ParseValue(e.kind().clone())
    }
}
