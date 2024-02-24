use std::collections::btree_set::BTreeSet;
use todo_txt::Task;

use super::ToDo;

/// Enum to represent the state of ToDo data (pending or done).
#[derive(Clone, Copy)]
pub enum ToDoData {
    Pending,
    Done,
}

impl ToDoData {
    /// Gets a reference to the specified ToDo data.
    ///
    /// # Arguments
    ///
    /// * `data` - The type of ToDo data to retrieve.
    pub fn get_data<'a>(&self, todo: &'a ToDo) -> &'a Vec<Task> {
        match self {
            Self::Pending => &todo.pending,
            Self::Done => &todo.done,
        }
    }

    /// Gets a mutable reference to the specified ToDo data (pending or done).
    ///
    /// # Arguments
    ///
    /// * `data` - The type of ToDo data to retrieve (Pending or Done).
    pub fn get_data_mut<'a>(&self, todo: &'a mut ToDo) -> &'a mut Vec<Task> {
        todo.version += 1;
        match self {
            Self::Pending => &mut todo.pending,
            Self::Done => &mut todo.done,
        }
    }
}

/// Enum to represent different categories.
#[derive(Clone, Copy, PartialEq)]
pub enum ToDoCategory {
    Projects,
    Contexts,
    Hashtags,
}

impl ToDoCategory {
    pub fn get_data<'a>(&self, task: &'a Task) -> &'a [String] {
        use ToDoCategory::*;
        match self {
            Projects => task.projects(),
            Contexts => task.contexts(),
            Hashtags => &task.hashtags,
        }
    }
}

pub struct ToDoState {
    pub active: Option<(ToDoData, usize)>,
    pub project_filters: BTreeSet<String>,
    pub context_filters: BTreeSet<String>,
    pub hashtag_filters: BTreeSet<String>,
}

impl ToDoState {
    pub fn get_category(&self, category: ToDoCategory) -> &BTreeSet<String> {
        use ToDoCategory::*;
        match category {
            Projects => &self.project_filters,
            Contexts => &self.context_filters,
            Hashtags => &self.hashtag_filters,
        }
    }

    pub fn get_mut_category(&mut self, category: ToDoCategory) -> &mut BTreeSet<String> {
        use ToDoCategory::*;
        match category {
            Projects => &mut self.project_filters,
            Contexts => &mut self.context_filters,
            Hashtags => &mut self.hashtag_filters,
        }
    }
}

impl Default for ToDoState {
    fn default() -> Self {
        Self {
            active: None,
            project_filters: BTreeSet::new(),
            context_filters: BTreeSet::new(),
            hashtag_filters: BTreeSet::new(),
        }
    }
}
