use std::{
    error::Error,
    fmt::{self, Display},
    num::ParseIntError,
};

pub type ToDoRes<T> = Result<T, ErrorToDo>;

#[derive(Debug)]
pub enum ErrorType {
    WidgetDoesNotExist,
    ParseValueError,
    ParseWidgetType,
    ParseNotStart,
    ParseNotEnd,
    ParseInvalidDirection,
    ActiveIsNotWidget,
}

#[derive(Debug)]
pub struct ErrorToDo {
    pub err_type: ErrorType,
    pub message: &'static str,
}

impl Error for ErrorToDo {}

impl Display for ErrorToDo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error<{:?}>: {}", self.err_type, self.message)
    }
}

impl ErrorToDo {
    pub fn new(err_type: ErrorType, message: &'static str) -> ErrorToDo {
        ErrorToDo { message, err_type }
    }
}

impl From<ParseIntError> for ErrorToDo {
    fn from(_val: ParseIntError) -> Self {
        ErrorToDo::new(
            ErrorType::ParseValueError,
            "Value must be in format unsigned integer.",
        )
    }
}
