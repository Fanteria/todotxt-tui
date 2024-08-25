pub mod autocomplete;
pub mod category_list;
pub mod parser;
pub mod task_list;
pub mod todo_state;
pub mod version;

pub use self::{
    autocomplete::autocomplete, category_list::CategoryList, parser::Parser, task_list::TaskList,
    todo_state::*,
};

use crate::config::{SetFinalDateType, Styles, ToDoConfig};
use chrono::{NaiveDate, Utc};
use std::{collections::btree_set::BTreeSet, str::FromStr};
use todo_txt::Task;
use version::Version;

/// Struct to manage ToDo tasks and theirs state.
pub struct ToDo {
    pub pending: Vec<Task>,
    pub done: Vec<Task>,
    version: Version,
    state: ToDoState,
    config: ToDoConfig,
    styles: Styles,
}

impl ToDo {
    /// Creates a new ToDo instance.
    ///
    /// # Arguments
    ///
    /// * `use_done` - A boolean indicating whether to include done tasks in the ToDo data.
    pub fn new(config: ToDoConfig, styles: Styles) -> Self {
        Self {
            pending: Vec::new(),
            done: Vec::new(),
            version: Version::default(),
            state: ToDoState::default(),
            config,
            styles,
        }
    }

    /// Moves data from another ToDo instance into this one.
    ///
    /// # Arguments
    ///
    /// * `other` - The other ToDo instance to move data from.
    pub fn move_data(&mut self, other: Self) {
        self.pending = other.pending;
        self.done = other.done;
        self.version.update_all();
    }

    /// Gets the current version of the ToDo data.
    /// Version is increased on every data change.
    pub fn get_version(&self) -> &Version {
        &self.version
    }

    /// Returns a mutable reference to the `Version` field within the current struct instance.
    pub fn get_version_mut(&mut self) -> &mut Version {
        &mut self.version
    }

    /// Gets the actual index of an item in the ToDo data without filters.
    ///
    /// # Arguments
    ///
    /// * `data` - The type of ToDo data (Pending or Done).
    /// * `index` - The index of the item in the filtered data.
    ///
    /// # Returns
    ///
    /// The actual index of the item in ToDo data without filtering.
    fn get_actual_index(&self, data: ToDoData, index: usize) -> Option<usize> {
        self.get_filtered_and_sorted(data).get_actual_index(index)
    }

    /// Retrieves the current date from UTC time without any modifications or adjustments
    /// applied to it.
    fn get_actual_date() -> NaiveDate {
        Utc::now().naive_utc().date()
    }

    /// Adds a new task to the ToDo list.
    ///
    /// # Arguments
    ///
    /// * `task` - The `Task` to be added to the ToDo list.
    pub fn add_task(&mut self, task: Task) {
        if task.finished {
            self.done.push(task);
            self.version.update(&ToDoData::Done);
        } else {
            self.pending.push(task);
            self.version.update(&ToDoData::Pending);
        }
    }

    /// Gets a filtered list of categories from the ToDo data.
    ///
    /// # Arguments
    ///
    /// * `category` - The type of category to retrieve.
    ///
    /// # Returns
    ///
    /// A `CategoryList` containing the filtered categories and their selection status.
    pub fn get_categories(&self, category: ToDoCategory) -> CategoryList {
        let tasks = if self.config.use_done {
            vec![&self.pending, &self.done]
        } else {
            vec![&self.pending]
        };

        let selected = self.state.get_category(category);
        CategoryList {
            vec: tasks
                .iter()
                .flat_map(|list| list.iter())
                .flat_map(|task| category.get_data(task).iter())
                .chain(self.state.get_category(category).keys())
                .collect::<BTreeSet<&String>>()
                .iter()
                .map(|item| (*item, selected.get(*item).cloned()))
                .collect(),
            styles: &self.styles,
        }
    }

    /// Moves a task from one section (Pending or Done) to the other.
    ///
    /// # Arguments
    ///
    /// * `data` - The type of ToDo data from which to move the task.
    /// * `index` - The index of the task to be moved in the specified data.
    pub fn move_task(&mut self, data: ToDoData, index: usize) {
        self.version.update_all();
        let index = match self.get_actual_index(data, index) {
            Some(index) => index,
            None => {
                log::warn!("Cannot move task Layout::get_actual_index is None");
                return;
            }
        };

        let move_task_logic = |from: &mut Vec<Task>, to: &mut Vec<_>| {
            if from.len() <= index {
                return;
            }
            let mut task = from.remove(index);
            if task.finished && self.config.delete_final_date {
                task.finish_date = None;
            }
            if !task.finished {
                match self.config.set_final_date {
                    SetFinalDateType::Override => task.finish_date = Some(Self::get_actual_date()),
                    SetFinalDateType::OnlyMissing if task.finish_date.is_none() => {
                        task.finish_date = Some(Self::get_actual_date())
                    }
                    _ => {}
                }
            }
            task.finished = !task.finished;
            to.push(task)
        };
        use ToDoData::*;
        match data {
            Pending => move_task_logic(&mut self.pending, &mut self.done),
            Done => move_task_logic(&mut self.done, &mut self.pending),
        };
        self.fix_active(index)
    }

    /// Toggles a filter for a specific category.
    ///
    /// # Arguments
    ///
    /// * `category` - The type of category to which the filter applies (Projects, Contexts, or Hashtags).
    /// * `filter` - The filter string to toggle.
    pub fn toggle_filter(
        &mut self,
        category: ToDoCategory,
        filter: &str,
        filter_state: FilterState,
    ) {
        self.state.set_filter(category, filter, filter_state)
    }

    fn get_filtered_tasks(&self, data: ToDoData) -> Vec<(usize, &Task)> {
        data.get_data(self)
            .iter()
            .enumerate()
            .filter(|(_, task)| self.state.filter_out(task))
            .collect()
    }

    /// Retrieves a filtered and sorted list of tasks based on active filters and sorting criteria.
    ///
    /// This function filters tasks according to the specified `ToDoData` and then sorts them based
    /// on the current sorting configuration.
    ///
    /// # Arguments
    ///
    /// * `data` - The `ToDoData` containing the filtering criteria.
    ///
    /// # Returns
    ///
    /// A `TaskList` containing the filtered and sorted tasks.
    pub fn get_filtered_and_sorted(&self, data: ToDoData) -> TaskList {
        let mut task_list = TaskList {
            vec: self.get_filtered_tasks(data),
            styles: &self.styles,
        };
        task_list.sort(data.get_sorting(&self.config));
        task_list
    }

    /// Adds a new task to the ToDo list using a task string.
    ///
    /// # Arguments
    ///
    /// * `task` - The task string to parse and add to the ToDo list.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error if the task string cannot be parsed.
    pub fn new_task(&mut self, task: &str) -> Result<(), todo_txt::Error> {
        let task = task.replace("due:today ", &format!("due:{}", Self::get_actual_date()));
        let task = task.replace("due: ", &format!("due:{}", Self::get_actual_date()));
        let mut task = Task::from_str(&task)?;
        if task.create_date.is_none() {
            task.create_date = Some(Self::get_actual_date());
        }
        if task.finished {
            self.done.push(task);
            self.version.update(&ToDoData::Done);
        } else {
            self.pending.push(task);
            self.version.update(&ToDoData::Pending);
        }
        Ok(())
    }

    /// Removes a task from the ToDo list.
    ///
    /// # Arguments
    ///
    /// * `data` - The type of ToDo data from which to remove the task.
    /// * `index` - The index of the task to be removed in the specified data.
    pub fn remove_task(&mut self, data: ToDoData, index: usize) {
        let index = self.get_actual_index(data, index);
        if let Some(index) = index {
            data.get_data_mut(self).remove(index);
            self.fix_active(index);
        } else {
            log::warn!("Layout::get_actual_index is None");
        }
    }

    /// Swaps the positions of two tasks in the ToDo list.
    ///
    /// # Arguments
    ///
    /// * `data` - The type of ToDo data (Pending or Done) in which to swap the tasks.
    /// * `from` - The index of the first task to be swapped.
    /// * `to` - The index of the second task to be swapped.
    pub fn swap_tasks(&mut self, data: ToDoData, from: usize, to: usize) {
        let from = self.get_actual_index(data, from);
        let to = self.get_actual_index(data, to);
        match (from, to) {
            (Some(from), Some(to)) => {
                data.get_data_mut(self).swap(from, to);
                if let Some((_, act_index)) = &mut self.state.active {
                    if *act_index == from {
                        *act_index = to;
                    } else if *act_index == to {
                        *act_index = from;
                    }
                }
            }
            _ => {
                log::warn!("Canot swap from or to is None")
            }
        }
    }

    /// Sets a task as the active task for potential editing.
    ///
    /// # Arguments
    ///
    /// * `data` - The type of ToDo data where the task is located.
    /// * `index` - The index of the task to be set as active in the specified data.
    pub fn set_active(&mut self, data: ToDoData, index: usize) {
        let index = self.get_actual_index(data, index);
        if let Some(index) = index {
            self.state.active = Some((data, index));
        } else {
            log::warn!("Layout::get_actual_index is None");
        }
    }

    /// Gets the currently active task for potential editing.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the active `Task`, or `None` if no task is active.
    pub fn get_active(&self) -> Option<&Task> {
        match self.state.active {
            Some((data, index)) => {
                let list = data.get_data(self);
                if index >= list.len() {
                    list.last()
                } else {
                    list.get(index)
                }
            }
            None => None,
        }
    }

    /// Updates the content of the active task.
    ///
    /// # Arguments
    ///
    /// * `task` - The updated task string.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error if the updated task string cannot be parsed.
    pub fn update_active(&mut self, task: &str) -> Result<(), todo_txt::Error> {
        if let Some((data, index)) = self.state.active {
            data.get_data_mut(self)[index] = Task::from_str(task)?;
        }
        Ok(())
    }

    /// Fixes the active task index in case of task movements or removals.
    ///
    /// This method is used internally to ensure that the active task index remains valid
    /// after tasks are moved or removed.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of a task that was moved or removed.
    fn fix_active(&mut self, index: usize) {
        if let Some((_, act_index)) = &mut self.state.active {
            log::trace!("act: {}, moved: {}", act_index, index);
            match index.cmp(act_index) {
                std::cmp::Ordering::Less => *act_index -= 1,
                std::cmp::Ordering::Equal => self.state.active = None,
                std::cmp::Ordering::Greater => {}
            }
        }
    }

    /// Gets the number of tasks in the specified ToDo data (Pending or Done).
    ///
    /// # Arguments
    ///
    /// * `data` - The type of ToDo data for which to count the tasks.
    ///
    /// # Returns
    ///
    /// The number of tasks in the specified ToDo data.
    pub fn len(&self, data: ToDoData) -> usize {
        self.get_filtered_and_sorted(data).len()
    }

    /// Returns a reference to the current state of the ToDo list.
    pub fn get_state(&self) -> &ToDoState {
        &self.state
    }

    /// Updates the ToDo list's state to the provided `ToDoState`.
    pub fn update_state(&mut self, state: ToDoState) {
        self.state = state
    }
}

impl Default for ToDo {
    fn default() -> Self {
        ToDo::new(ToDoConfig::default(), Styles::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::naive::NaiveDate;
    use std::error::Error;
    use todo_txt::Priority;

    fn example_todo() -> ToDo {
        let mut todo = ToDo::default();

        let mut task = Task::from_str("measure space for 1 +project1 @context1 #hashtag1").unwrap();
        task.finished = true;
        task.priority = Priority::from(0);
        task.create_date = Some(NaiveDate::from_ymd_opt(2023, 4, 30).unwrap());
        task.finish_date = Some(NaiveDate::from_ymd_opt(2023, 5, 21).unwrap());
        task.due_date = Some(NaiveDate::from_ymd_opt(2023, 6, 30).unwrap());
        todo.add_task(task);

        let mut task = Task::from_str("measure space for 2 +project2 @context2").unwrap();
        task.create_date = Some(NaiveDate::from_ymd_opt(2023, 4, 30).unwrap());
        task.due_date = Some(NaiveDate::from_ymd_opt(2023, 6, 30).unwrap());
        todo.add_task(task);

        let mut task = Task::from_str("measure space for 3 +project3 @context3").unwrap();
        task.priority = Priority::from(2);
        task.create_date = Some(NaiveDate::from_ymd_opt(2023, 4, 30).unwrap());
        task.due_date = Some(NaiveDate::from_ymd_opt(2023, 6, 30).unwrap());
        todo.add_task(task);

        let mut task = Task::from_str("measure space for +project2 @context3 #hashtag1").unwrap();
        task.due_date = Some(NaiveDate::from_ymd_opt(2023, 6, 30).unwrap());
        todo.add_task(task);

        let mut task = Task::from_str("measure space for 5 +project3 @context3 #hashtag2").unwrap();
        task.finished = true;
        task.due_date = Some(NaiveDate::from_ymd_opt(2023, 6, 30).unwrap());
        todo.add_task(task);

        let mut task = Task::from_str("measure space for 6 +project3 @context2 #hashtag2").unwrap();
        task.due_date = Some(NaiveDate::from_ymd_opt(2023, 6, 30).unwrap());
        todo.add_task(task);

        todo
    }

    #[test]
    fn test_add_task() {
        let mut todo = example_todo();
        todo.config.use_done = true;

        assert_eq!(todo.done.len(), 2);
        assert_eq!(todo.pending.len(), 4);

        assert_eq!(todo.done[0].priority, 0);
        assert!(todo.done[0].create_date.is_some());
        assert!(todo.done[0].finish_date.is_some());
        assert!(todo.done[0].finished);
        assert_eq!(todo.done[0].threshold_date, None);
        assert!(todo.done[0].due_date.is_some());
        assert_eq!(todo.done[0].contexts().len(), 1);
        assert_eq!(todo.done[0].projects().len(), 1);
        assert_eq!(todo.done[0].hashtags.len(), 1);

        println!("{:#?}", todo.pending[0]);

        assert!(todo.pending[0].priority.is_lowest());
        assert!(todo.pending[0].create_date.is_some());
        assert!(todo.pending[0].finish_date.is_none());
        assert!(!todo.pending[0].finished);
        assert_eq!(todo.pending[0].threshold_date, None);
        assert!(todo.pending[0].due_date.is_some());
        assert_eq!(todo.pending[0].contexts().len(), 1);
        assert_eq!(todo.pending[0].projects().len(), 1);
        assert_eq!(todo.pending[0].hashtags.len(), 0);

        assert_eq!(todo.pending[1].priority, 2);
        assert!(todo.pending[1].create_date.is_some());
        assert!(todo.pending[1].finish_date.is_none());
        assert!(!todo.pending[1].finished);
        assert_eq!(todo.pending[1].threshold_date, None);
        assert!(todo.pending[1].due_date.is_some());
        assert_eq!(todo.pending[1].contexts().len(), 1);
        assert_eq!(todo.pending[1].projects().len(), 1);
        assert_eq!(todo.pending[1].hashtags.len(), 0);
    }

    fn create_vec(items: &[String]) -> Vec<(&String, Option<FilterState>)> {
        let mut vec: Vec<(&String, Option<FilterState>)> = Vec::new();
        items.iter().for_each(|item| {
            vec.push((item, None));
        });
        vec
    }

    #[test]
    fn test_categeries_list() -> Result<(), Box<dyn Error>> {
        let mut todo = example_todo();
        assert_eq!(
            todo.get_categories(ToDoCategory::Projects).vec,
            create_vec(&[String::from("project2"), String::from("project3")])
        );
        assert_eq!(
            todo.get_categories(ToDoCategory::Contexts).vec,
            create_vec(&[String::from("context2"), String::from("context3")])
        );
        assert_eq!(
            todo.get_categories(ToDoCategory::Hashtags).vec,
            create_vec(&[String::from("hashtag1"), String::from("hashtag2")])
        );

        todo.config.use_done = true;
        assert_eq!(
            todo.get_categories(ToDoCategory::Projects).vec,
            create_vec(&[
                String::from("project1"),
                String::from("project2"),
                String::from("project3"),
            ])
        );
        assert_eq!(
            todo.get_categories(ToDoCategory::Contexts).vec,
            create_vec(&[
                String::from("context1"),
                String::from("context2"),
                String::from("context3"),
            ])
        );
        assert_eq!(
            todo.get_categories(ToDoCategory::Hashtags).vec,
            create_vec(&[String::from("hashtag1"), String::from("hashtag2")])
        );

        Ok(())
    }

    #[test]
    fn test_filtering() -> Result<(), Box<dyn Error>> {
        let mut todo = ToDo::default();
        todo.add_task(Task::from_str("task 1").unwrap());
        todo.add_task(Task::from_str("task 2 +project1").unwrap());
        todo.add_task(Task::from_str("task 3 +project1 +project2").unwrap());
        todo.add_task(Task::from_str("task 4 +project1 +project3").unwrap());
        todo.add_task(Task::from_str("task 5 +project1 +project2 +project3").unwrap());
        todo.add_task(Task::from_str("task 6 +project3 @context2 #hashtag2 #hashtag1").unwrap());
        todo.add_task(Task::from_str("task 7 +project2 @context1 #hashtag1 #hashtag2").unwrap());
        todo.add_task(Task::from_str("task 8 +project2 @context2").unwrap());
        todo.add_task(Task::from_str("task 9 +projects3 @context3").unwrap());
        todo.add_task(Task::from_str("task 10 +project2 @context3 #hashtag1 #hashtag2").unwrap());
        todo.add_task(Task::from_str("task 11 +project3 @context3 #hashtag2 #hashtag3").unwrap());
        todo.add_task(Task::from_str("task 12 +project3 @context2 #hashtag2").unwrap());

        let filtered = todo.get_filtered_and_sorted(ToDoData::Pending);
        assert_eq!(filtered.len(), 12);

        todo.state
            .project_filters
            .insert(String::from("project9999"), FilterState::Select);
        let filtered = todo.get_filtered_and_sorted(ToDoData::Pending);
        assert_eq!(filtered.len(), 0);

        todo.state.project_filters.clear();
        todo.state
            .project_filters
            .insert(String::from("project1"), FilterState::Select);
        let filtered = todo.get_filtered_and_sorted(ToDoData::Pending);
        assert_eq!(filtered.len(), 4);
        assert_eq!(filtered[0].subject, "task 2 +project1");
        assert_eq!(filtered[1].subject, "task 3 +project1 +project2");
        assert_eq!(filtered[2].subject, "task 4 +project1 +project3");
        assert_eq!(filtered[3].subject, "task 5 +project1 +project2 +project3");

        todo.state
            .project_filters
            .insert(String::from("project2"), FilterState::Select);
        let filtered = todo.get_filtered_and_sorted(ToDoData::Pending);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].subject, "task 3 +project1 +project2");
        assert_eq!(filtered[1].subject, "task 5 +project1 +project2 +project3");

        todo.state
            .project_filters
            .insert(String::from("project3"), FilterState::Select);
        let filtered = todo.get_filtered_and_sorted(ToDoData::Pending);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].subject, "task 5 +project1 +project2 +project3");

        todo.state
            .project_filters
            .insert(String::from("project1"), FilterState::Select);
        let filtered = todo.get_filtered_and_sorted(ToDoData::Pending);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].subject, "task 5 +project1 +project2 +project3");

        todo.state.project_filters.clear();
        todo.state
            .context_filters
            .insert(String::from("context1"), FilterState::Select);
        let filtered = todo.get_filtered_and_sorted(ToDoData::Pending);
        assert_eq!(filtered.len(), 1);
        assert_eq!(
            filtered[0].subject,
            "task 7 +project2 @context1 #hashtag1 #hashtag2"
        );

        Ok(())
    }

    #[test]
    fn actual_consistency_move() {
        let mut todo = example_todo();
        todo.set_active(ToDoData::Pending, 2);
        let subject = todo.get_active().unwrap().subject.clone();
        // Item after
        todo.move_task(ToDoData::Pending, 3);
        assert_eq!(todo.get_active().unwrap().subject, subject);

        // Item before
        todo.move_task(ToDoData::Pending, 0);
        assert_eq!(todo.get_active().unwrap().subject, subject);

        // Active item
        todo.move_task(ToDoData::Pending, 1);
        assert!(todo.get_active().is_none());
    }

    #[test]
    fn actual_consistency_remove() {
        let mut todo = example_todo();
        todo.set_active(ToDoData::Pending, 2);
        let subject = todo.get_active().unwrap().subject.clone();
        // Item after
        todo.remove_task(ToDoData::Pending, 3);
        assert_eq!(todo.get_active().unwrap().subject, subject);

        // Item before
        todo.remove_task(ToDoData::Pending, 0);
        assert_eq!(todo.get_active().unwrap().subject, subject);

        // Active item
        todo.remove_task(ToDoData::Pending, 1);
        assert!(todo.get_active().is_none());
    }

    #[test]
    fn actual_consistency_swap() {
        let mut todo = example_todo();
        todo.set_active(ToDoData::Pending, 2);
        let subject = todo.get_active().unwrap().subject.clone();
        // Item outside
        todo.swap_tasks(ToDoData::Pending, 0, 1);
        assert_eq!(todo.get_active().unwrap().subject, subject);

        // Item from
        todo.swap_tasks(ToDoData::Pending, 2, 0);
        assert_eq!(todo.get_active().unwrap().subject, subject);

        // Item to
        todo.swap_tasks(ToDoData::Pending, 1, 2);
        assert_eq!(todo.get_active().unwrap().subject, subject);
    }

    #[test]
    fn move_data() {
        let todo = example_todo();
        let mut empty = ToDo::default();
        assert!(empty.pending.is_empty());
        assert!(empty.done.is_empty());
        empty.move_data(example_todo());
        assert_eq!(todo.pending, empty.pending);
        assert_eq!(todo.done, empty.done);
    }

    #[test]
    fn version() {
        let mut todo = ToDo::default();
        assert!(todo.get_version().is_actual(0, &ToDoData::Pending));
        assert!(todo.get_version().is_actual(0, &ToDoData::Done));
        todo.move_data(example_todo());
        println!("{:?}", todo.get_version());
        assert!(todo.get_version().is_actual(1, &ToDoData::Pending));
        assert!(todo.get_version().is_actual(1, &ToDoData::Done));
        todo.move_task(ToDoData::Done, 1);
        assert!(todo.get_version().is_actual(2, &ToDoData::Pending));
        assert!(todo.get_version().is_actual(2, &ToDoData::Done));
        todo.add_task(Task::from_str("Some simple task").unwrap());
        assert!(todo.get_version().is_actual(3, &ToDoData::Pending));
        assert!(todo.get_version().is_actual(2, &ToDoData::Done));
        todo.add_task(Task::from_str("x Some simple task").unwrap());
        assert!(todo.get_version().is_actual(3, &ToDoData::Pending));
        assert!(todo.get_version().is_actual(3, &ToDoData::Done));
    }

    #[test]
    fn toggle_filter() {
        let mut todo = example_todo();
        assert!(todo.state.project_filters.is_empty());
        todo.toggle_filter(ToDoCategory::Projects, "project1", FilterState::Select);
        assert_eq!(
            todo.state.project_filters.get("project1"),
            Some(&FilterState::Select)
        );
        assert_eq!(todo.state.project_filters.len(), 1);
        todo.toggle_filter(ToDoCategory::Projects, "project1", FilterState::Select);
        assert!(todo.state.project_filters.is_empty());

        todo.toggle_filter(ToDoCategory::Contexts, "context1", FilterState::Select);
        assert_eq!(
            todo.state.context_filters.get("context1"),
            Some(&FilterState::Select)
        );
        assert_eq!(todo.state.context_filters.len(), 1);
        todo.toggle_filter(ToDoCategory::Contexts, "context1", FilterState::Select);
        assert!(todo.state.context_filters.is_empty());

        todo.toggle_filter(ToDoCategory::Hashtags, "hashtag1", FilterState::Select);
        assert_eq!(
            todo.state.hashtag_filters.get("hashtag1"),
            Some(&FilterState::Select)
        );
        assert_eq!(todo.state.hashtag_filters.len(), 1);
        todo.toggle_filter(ToDoCategory::Hashtags, "hashtag1", FilterState::Select);
        assert!(todo.state.hashtag_filters.is_empty());
    }

    #[test]
    fn new_task() -> Result<(), todo_txt::Error> {
        let mut todo = ToDo::default();
        todo.new_task("Some pending task")?;
        assert_eq!(todo.pending.len(), 1);
        assert_eq!(todo.pending[0].subject, "Some pending task");
        todo.new_task("x Some done task")?;
        assert_eq!(todo.done.len(), 1);
        assert_eq!(todo.done[0].subject, "Some done task");

        Ok(())
    }

    #[test]
    fn update_active() -> Result<(), todo_txt::Error> {
        let mut todo = example_todo();
        todo.state.active = Some((ToDoData::Pending, 0));
        todo.update_active("New subject")?;
        assert_eq!(todo.pending[0].subject, "New subject");

        todo.state.active = Some((ToDoData::Done, 0));
        todo.update_active("New done subject")?;
        assert_eq!(todo.done[0].subject, "New done subject");

        Ok(())
    }

    #[test]
    fn update_finish_date() {
        let mut todo = example_todo();

        todo.config.set_final_date = SetFinalDateType::OnlyMissing;
        assert_eq!(todo.pending[0].finish_date, None);
        todo.move_task(ToDoData::Pending, 0);
        assert_eq!(
            todo.done.last().unwrap().finish_date,
            Some(ToDo::get_actual_date())
        );

        todo.config.set_final_date = SetFinalDateType::Never;
        assert_eq!(todo.pending[0].finish_date, None);
        todo.move_task(ToDoData::Pending, 0);
        assert_eq!(todo.done.last().unwrap().finish_date, None);

        todo.config.set_final_date = SetFinalDateType::Override;
        todo.pending[0].finish_date = Some(NaiveDate::from_ymd_opt(2023, 4, 30).unwrap());
        todo.move_task(ToDoData::Pending, 0);
        assert_eq!(
            todo.done.last().unwrap().finish_date,
            Some(ToDo::get_actual_date())
        );

        todo.config.delete_final_date = true;
        todo.done[0].finish_date = Some(NaiveDate::from_ymd_opt(2023, 4, 30).unwrap());
        todo.move_task(ToDoData::Done, 0);
        assert_eq!(todo.pending.last().unwrap().finish_date, None);

        todo.config.delete_final_date = false;
        let date = Some(NaiveDate::from_ymd_opt(2023, 4, 30).unwrap());
        todo.done[0].finish_date = date;
        todo.move_task(ToDoData::Done, 0);
        assert_eq!(todo.pending.last().unwrap().finish_date, date);
    }
}
