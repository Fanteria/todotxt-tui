macro_rules! some_or_return {
    ($message:expr) => {
        match $message {
            Some(s) => s,
            None => return,
        }
    };
}

pub(crate) use some_or_return;
