use std::{error, fmt};

#[derive(Debug)]
#[allow(dead_code)]
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

impl error::Error for ErrorToDo {}

impl fmt::Display for ErrorToDo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error<{:?}>: {}", self.err_type, self.message)
    }
}

impl ErrorToDo {
    pub fn new(err_type: ErrorType, message: &'static str) -> ErrorToDo {
        ErrorToDo { message, err_type }
    }
}
