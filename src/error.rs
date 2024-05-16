use std::path::PathBuf;

/// Define a custom result type for ToDo related operations.
pub type ToDoRes<T> = Result<T, ToDoError>;

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
}

#[derive(Debug, thiserror::Error)]
#[error("IO error file: {path}, error: {err}")]
pub struct ToDoIoError {
    pub path: PathBuf,
    pub err: std::io::Error,
}

impl PartialEq for ToDoIoError {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.err.kind() == other.err.kind()
    }
}
