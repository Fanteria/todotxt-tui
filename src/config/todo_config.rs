use super::Config;
use crate::todo::task_list::TaskSort;

pub enum SetFinalDateType {
    Override,
    OnlyMissing,
    Never,
}

pub struct ToDoConfig {
    pub use_done: bool,
    pub pending_sort: TaskSort,
    pub done_sort: TaskSort,
    pub delete_final_date: bool,
    pub set_final_date: SetFinalDateType,
}

impl ToDoConfig {
    pub fn new(config: &Config) -> Self {
        Self {
            use_done: false, // TODO add to config
            pending_sort: config.get_pending_sort(),
            done_sort: config.get_done_sort(),
            delete_final_date: true, // TODO add to config
            set_final_date: SetFinalDateType::OnlyMissing, // TODO add to config
        }
    }
}
