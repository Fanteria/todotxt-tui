use std::{
    env::VarError,
    path::{Path, PathBuf},
    result::Result as stdResult,
};

/// Define a custom result type for ToDo related operations.
pub type Result<T> = stdResult<T, ToDoError>;

/// Enum representing ToDo-related errors.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ToDoError {
    #[error("Selected widgent is not in layout")]
    WidgetDoesNotExist,
    #[error("Value cannot be parsed: {0}")]
    ParseValue(#[from] std::num::ParseIntError),
    #[error("Value can constraint only unsigned integer and % not {0}")]
    ParseUnknownValue(String),
    #[error("Unknown widget type: {0}")]
    ParseWidgetType(String),
    #[error("There must be almost one container.")]
    ParseNotStart,
    #[error("All containers must be closed.")]
    ParseNotEnd,
    #[error("Unknown text before start of the container '{0}'")]
    ParseUnknowBeforeContainer(String),
    #[error("Direction '{0}' is invalid.")]
    ParseInvalidDirection(String),
    #[error("Style '{0}' is invalid")]
    ParseTextStyle(String),
    #[error("Style list '{0}' does not contain at least one item")]
    ParseTextStyleList(String),
    #[error("Modifier '{0}' is invalid.")]
    ParseTextModifier(String),
    #[error("Block '{0}' have escape on the end.")]
    ParseBlockEscapeOnEnd(String),
    #[error("Block '{0}' is not closed.")]
    ParseBlockNotClosed(String),
    #[error("Variable '{0}' is not closed.")]
    ParseVariableNotClosed(String),
    #[error("Block '{0}' constraint empty variable name.")]
    EmptyVariableName(String),
    #[error("Invalid state, active container is not widget.")]
    ActiveIsNotWidget,
    #[error("{0}")]
    IOoperationFailed(#[from] ToDoIoError),
    #[error("{0}")]
    IOFailed(#[from] IOError),
    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),
    #[error("TOML deserialization error: {}", .0.to_string())]
    TomlDesError(#[from] toml::de::Error),
    #[error("String '{0}' is not valid color.")]
    ColorSerializationFailed(String),
    #[error("Var error: {0}")]
    VarError(#[from] VarError),
    #[error("Cannot load configuration: {}", .0.to_string())]
    ConfigLoadError(#[from] TwelfError),
    #[error("Cannot parse event entry: {0}")]
    CannotParseEventEntry(String),
    #[error("Cannot parse UI event: {0}")]
    CannotParseUIEvent(String),
    #[error("Cannot parse custom category style: {0}")]
    CustomCategoryStyleParseFailed(&'static str),
    #[error("notify")]
    NotifyError(#[from] NotifyError),
}

#[derive(Debug, thiserror::Error)]
#[error("IOError: {0}")]
pub struct NotifyError(#[from] pub notify::Error);

impl PartialEq for NotifyError {
    fn eq(&self, _: &Self) -> bool {
        // TODO implement
        false
    }
}

#[derive(Debug, thiserror::Error)]
#[error("IO error file: {path}, error: {err}")]
pub struct ToDoIoError {
    pub path: PathBuf,
    pub err: std::io::Error,
}

impl ToDoIoError {
    pub fn new(path: impl AsRef<Path>, err: std::io::Error) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            err,
        }
    }
}

impl PartialEq for ToDoIoError {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.err.kind() == other.err.kind()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("IOError: {0}")]
pub struct IOError(#[from] pub std::io::Error);

impl PartialEq for IOError {
    fn eq(&self, other: &Self) -> bool {
        self.0.kind() == other.0.kind()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct TwelfError(#[from] pub twelf::Error);

impl PartialEq for TwelfError {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_string() == other.0.to_string()
    }
}
