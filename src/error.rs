use std::{
    env::VarError,
    path::{Path, PathBuf},
    result::Result as stdResult,
    string::FromUtf8Error,
};

/// Define a custom result type for ToDo related operations.
pub type Result<T> = stdResult<T, ToDoError>;

/// Enum representing ToDo-related errors.
#[derive(Debug, thiserror::Error)]
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
    #[error("IO error file: {0}, error: {1}")]
    IOoperationFailed(PathBuf, #[source] std::io::Error),
    #[error("IOError: {0}")]
    IOFailed(#[from] std::io::Error),
    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),
    #[error("TOML deserialization error: {}", .0.to_string())]
    TomlDesError(#[from] toml::de::Error),
    #[error("String '{0}' is not valid color.")]
    ColorSerializationFailed(String),
    #[error("Var error: {0}")]
    VarError(#[from] VarError),
    #[error("Cannot load configuration: {0}")]
    ConfigLoadError(#[from] twelf::Error),
    #[error("Cannot parse event entry: {0}")]
    CannotParseEventEntry(String),
    #[error("Cannot parse UI event: {0}")]
    CannotParseUIEvent(String),
    #[error("Cannot parse custom category style: {0}")]
    CustomCategoryStyleParseFailed(&'static str),
    #[error("Notify {0}")]
    NotifyError(#[from] notify::Error),
    #[error("Todo txt parsing failed {0}")]
    Todotxt(#[from] todo_txt::Error),
    #[error("Failed to run hook {0:?}, stderr: {1}")]
    HookCommandFailed(PathBuf, String),
    #[error("Failed to parse hook stderr {0}")]
    HookFailedToParseError(#[source] FromUtf8Error),
    #[error("Failed to parse hook stdout {0}")]
    HookFailedToParseStdout(#[source] FromUtf8Error),
}

impl ToDoError {
    /// Add path to IO operation failed error.
    pub fn io_operation_failed(path: impl AsRef<Path>, err: std::io::Error) -> Self {
        Self::IOoperationFailed(path.as_ref().to_path_buf(), err)
    }
}
