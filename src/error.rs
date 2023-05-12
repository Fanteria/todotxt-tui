// use std::{error, fmt};
//
// #[derive(Debug)]
// pub enum Type {
//     ImpossigleLayout,
// }
//
// #[derive(Debug)]
// pub struct Error {
//     pub err_type: Type,
//     pub message: &'static str,
// }
//
// impl error::Error for Error {}
//
// impl fmt::Display for Error {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Error<{:?}>: {}", self.err_type, self.message)
//     }
// }
//
// impl Error {
//     pub fn new(err_type: Type, message: &'static str) -> Error {
//         Error { message, err_type }
//     }
// }
