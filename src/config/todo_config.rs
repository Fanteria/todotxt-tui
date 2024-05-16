use super::Config;
use crate::todo::task_list::TaskSort;

pub struct ToDoConfig {
    pub use_done: bool,
    pub pending_sort: TaskSort,
    pub done_sort: TaskSort,
}

impl ToDoConfig {
    pub fn new(config: &Config) -> Self {
        Self {
            use_done: false, // TODO add to config
            pending_sort: config.get_pending_sort(),
            done_sort: config.get_done_sort(),
        }
    }
}
