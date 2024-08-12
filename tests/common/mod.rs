use std::{env, path::PathBuf};

pub fn get_test_dir() -> String {
    env::var("TODO_TUI_TEST_DIR").unwrap()
}

pub fn get_test_file(name: &str) -> PathBuf {
    let path = PathBuf::from(get_test_dir()).join(name);
    log::trace!("Get test file {path:?}");
    path
}
