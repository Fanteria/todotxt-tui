use std::{
    error::Error,
    fmt::{self, Display},
};

pub type ToDoRes<T> = Result<T, ErrorToDo>;

#[derive(Debug)]
pub enum ErrorType {
    ImpossigleLayout,
    WidgetDoesNotExist,
    ActualIsNotWidget,
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
