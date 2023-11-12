pub mod category_list;
pub mod task_list;
pub mod parser;
pub use self::{category_list::CategoryList, task_list::TaskList, parser::Parser};

use chrono::Utc;
use std::{collections::btree_set::BTreeSet, convert::From, str::FromStr};
use todo_txt::Task;

/// Type alias for a tuple representing filter data.
type FilterData<'a> = (&'a BTreeSet<String>, fn(&'a Task) -> &'a [String]);

/// Enum to represent the state of ToDo data (pending or done).
#[derive(Clone, Copy)]
pub enum ToDoData {
    Pending,
    Done,
}
use ToDoData::*;

/// Enum to represent different categories.
#[derive(Clone, Copy, PartialEq)]
pub enum ToDoCategory {
    Projects,
    Contexts,
    Hashtags,
}
use ToDoCategory::*;

/// Struct to manage ToDo tasks and theirs state.
#[derive(Default)]
pub struct ToDo {
    pub pending: Vec<Task>,
    pub done: Vec<Task>,
    use_done: bool,
    active: Option<(ToDoData, usize)>,
    version: usize,
    project_filters: BTreeSet<String>,
    context_filters: BTreeSet<String>,
    hashtag_filters: BTreeSet<String>,
}

impl ToDo {
    /// Creates a new ToDo instance.
    ///
    /// # Arguments
    ///
    /// * `use_done` - A boolean indicating whether to include done tasks in the ToDo data.
    pub fn new(use_done: bool) -> Self {
        Self {
            pending: Vec::new(),
            done: Vec::new(),
            use_done,
            active: None,
            version: 0,
            project_filters: BTreeSet::new(),
            context_filters: BTreeSet::new(),
            hashtag_filters: BTreeSet::new(),
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
        self.version += 1;
    }

    /// Gets a reference to the specified ToDo data.
    ///
    /// # Arguments
    ///
    /// * `data` - The type of ToDo data to retrieve.
    pub fn get_data(&self, data: ToDoData) -> &Vec<Task> {
        match data {
            Pending => &self.pending,
            Done => &self.done,
        }
    }

    /// Gets a mutable reference to the specified ToDo data (pending or done).
    ///
    /// # Arguments
    ///
    /// * `data` - The type of ToDo data to retrieve (Pending or Done).
    fn get_data_mut(&mut self, data: ToDoData) -> &mut Vec<Task> {
        self.version += 1;
        match data {
            Pending => &mut self.pending,
            Done => &mut self.done,
        }
    }

    /// Gets the current version of the ToDo data.
    /// Version is increased on every data change.
    pub fn get_version(&self) -> usize {
        self.version
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
    fn get_actual_index(&self, data: ToDoData, index: usize) -> usize {
        self.get_filtered(data).get_actual_index(index)
    }

    /// Adds a new task to the ToDo list.
    ///
    /// # Arguments
    ///
    /// * `task` - The `Task` to be added to the ToDo list.
    pub fn add_task(&mut self, task: Task) {
        self.version += 1;
        if task.finished {
            self.done.push(task);
        } else {
            self.pending.push(task);
        }
    }

    /// Gets a filtered list of categories from the ToDo data.
    ///
    /// # Arguments
    ///
    /// * `category` - The type of category to retrieve (Projects, Contexts, or Hashtags).
    ///
    /// # Returns
    ///
    /// A `CategoryList` containing the filtered categories and their selection status.
    fn get_btree<'a>(
        tasks: Vec<&'a Vec<Task>>,
        f: fn(&Task) -> &[String],
        selected: &BTreeSet<String>,
    ) -> CategoryList<'a> {
        let mut btree = BTreeSet::<&String>::new();
        tasks.iter().for_each(|list| {
            list.iter().for_each(|task| {
                f(task).iter().for_each(|project| {
                    btree.insert(project);
                })
            })
        });
        CategoryList(
            btree
                .iter()
                .map(|item| (*item, selected.contains(*item)))
                .collect(),
        )
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
        let get_btree_done_switch = |f, selected| {
            Self::get_btree(
                if self.use_done {
                    vec![&self.pending, &self.done]
                } else {
                    vec![&self.pending]
                },
                f,
                selected,
            )
        };
        match category {
            Projects => get_btree_done_switch(|t| t.projects(), &self.project_filters),
            Contexts => get_btree_done_switch(|t| t.contexts(), &self.context_filters),
            Hashtags => get_btree_done_switch(|t| &t.hashtags, &self.hashtag_filters),
        }
    }

    /// Moves a task from one section (Pending or Done) to the other.
    ///
    /// # Arguments
    ///
    /// * `data` - The type of ToDo data from which to move the task.
    /// * `index` - The index of the task to be moved in the specified data.
    pub fn move_task(&mut self, data: ToDoData, index: usize) {
        self.version += 1;
        let index = self.get_actual_index(data, index);

        let move_task_logic = |from: &mut Vec<Task>, to: &mut Vec<_>| {
            if from.len() <= index {
                return;
            }
            let mut task = from.remove(index);
            task.finished = !task.finished;
            to.push(task)
        };
        match data {
            Pending => {
                move_task_logic(&mut self.pending, &mut self.done);
            }
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
    pub fn toggle_filter(&mut self, category: ToDoCategory, filter: &str) {
        let filter_set = match category {
            Projects => &mut self.project_filters,
            Contexts => &mut self.context_filters,
            Hashtags => &mut self.hashtag_filters,
        };
        if !filter_set.insert(String::from(filter)) {
            filter_set.remove(filter);
        }
    }

    /// Gets a filtered list of tasks based on active filters.
    ///
    /// # Arguments
    ///
    /// * `data` - The type of ToDo data to filter.
    ///
    /// # Returns
    ///
    /// A `TaskList` containing the filtered tasks.
    pub fn get_filtered(&self, data: ToDoData) -> TaskList {
        fn get_filtered_tasks<'a>(tasks: &'a [Task], filters: &[FilterData<'a>]) -> TaskList<'a> {
            TaskList(
                tasks
                    .iter()
                    .enumerate()
                    .filter(|task| {
                        filters.iter().all(|filter| {
                            filter.0.iter().all(|item| filter.1(task.1).contains(item))
                        })
                    })
                    .collect(),
            )
        }
        get_filtered_tasks(
            self.get_data(data),
            &[
                (&self.project_filters, |t| t.projects()),
                (&self.context_filters, |t| t.contexts()),
                (&self.hashtag_filters, |t| &t.hashtags),
            ],
        )
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
        self.version += 1;
        let mut task = Task::from_str(task)?;
        if task.create_date.is_none() {
            task.create_date = Some(Utc::now().naive_utc().date());
        }
        if task.finished {
            self.done.push(task);
        } else {
            self.pending.push(task);
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
        self.get_data_mut(data).remove(index);
        self.fix_active(index);
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
        self.get_data_mut(data).swap(from, to);
        if let Some((_, act_index)) = &mut self.active {
            if *act_index == from {
                *act_index = to;
            } else if *act_index == to {
                *act_index = from;
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
        self.active = Some((data, index));
    }

    /// Gets the currently active task for potential editing.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the active `Task`, or `None` if no task is active.
    pub fn get_active(&self) -> Option<&Task> {
        match self.active {
            Some((data, index)) => Some(&self.get_data(data)[index]),
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
        if let Some((data, index)) = self.active {
            self.get_data_mut(data)[index] = Task::from_str(task)?;
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
        if let Some((_, act_index)) = &mut self.active {
            log::trace!("act: {}, moved: {}", act_index, index);
            match index.cmp(act_index) {
                std::cmp::Ordering::Less => *act_index -= 1,
                std::cmp::Ordering::Equal => self.active = None,
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
        self.get_filtered(data).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::naive::NaiveDate;
    use std::error::Error;
    use todo_txt::Priority;

    fn example_todo(use_done: bool) -> ToDo {
        let mut todo = ToDo::new(use_done);

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
        let todo = example_todo(true);

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

    fn create_vec(items: &[String]) -> Vec<(&String, bool)> {
        let mut vec: Vec<(&String, bool)> = Vec::new();
        items.iter().for_each(|item| {
            vec.push((item, false));
        });
        vec
    }

    #[test]
    fn test_categeries_list() -> Result<(), Box<dyn Error>> {
        let mut todo = example_todo(false);
        assert_eq!(
            todo.get_categories(ToDoCategory::Projects).0,
            create_vec(&[String::from("project2"), String::from("project3")])
        );
        assert_eq!(
            todo.get_categories(ToDoCategory::Contexts).0,
            create_vec(&[String::from("context2"), String::from("context3")])
        );
        assert_eq!(
            todo.get_categories(ToDoCategory::Hashtags).0,
            create_vec(&[String::from("hashtag1"), String::from("hashtag2")])
        );

        todo.use_done = true;
        assert_eq!(
            todo.get_categories(ToDoCategory::Projects).0,
            create_vec(&[
                String::from("project1"),
                String::from("project2"),
                String::from("project3"),
            ])
        );
        assert_eq!(
            todo.get_categories(ToDoCategory::Contexts).0,
            create_vec(&[
                String::from("context1"),
                String::from("context2"),
                String::from("context3"),
            ])
        );
        assert_eq!(
            todo.get_categories(ToDoCategory::Hashtags).0,
            create_vec(&[String::from("hashtag1"), String::from("hashtag2")])
        );

        Ok(())
    }

    #[test]
    fn test_filtering() -> Result<(), Box<dyn Error>> {
        let mut todo = ToDo::new(false);
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

        let filtered = todo.get_filtered(ToDoData::Pending);
        assert_eq!(filtered.len(), 12);

        todo.project_filters.insert(String::from("project9999"));
        let filtered = todo.get_filtered(ToDoData::Pending);
        assert_eq!(filtered.len(), 0);

        todo.project_filters.clear();
        todo.project_filters.insert(String::from("project1"));
        let filtered = todo.get_filtered(ToDoData::Pending);
        assert_eq!(filtered.len(), 4);
        assert_eq!(filtered[0].subject, "task 2 +project1");
        assert_eq!(filtered[1].subject, "task 3 +project1 +project2");
        assert_eq!(filtered[2].subject, "task 4 +project1 +project3");
        assert_eq!(filtered[3].subject, "task 5 +project1 +project2 +project3");

        todo.project_filters.insert(String::from("project2"));
        let filtered = todo.get_filtered(ToDoData::Pending);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].subject, "task 3 +project1 +project2");
        assert_eq!(filtered[1].subject, "task 5 +project1 +project2 +project3");

        todo.project_filters.insert(String::from("project3"));
        let filtered = todo.get_filtered(ToDoData::Pending);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].subject, "task 5 +project1 +project2 +project3");

        todo.project_filters.insert(String::from("project1"));
        let filtered = todo.get_filtered(ToDoData::Pending);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].subject, "task 5 +project1 +project2 +project3");

        todo.project_filters.clear();
        todo.context_filters.insert(String::from("context1"));
        let filtered = todo.get_filtered(ToDoData::Pending);
        assert_eq!(filtered.len(), 1);
        assert_eq!(
            filtered[0].subject,
            "task 7 +project2 @context1 #hashtag1 #hashtag2"
        );

        Ok(())
    }

    #[test]
    fn actual_consistency_move() {
        let mut todo = example_todo(false);
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
        let mut todo = example_todo(false);
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
        let mut todo = example_todo(false);
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
        let todo = example_todo(false);
        let mut empty = ToDo::default();
        assert!(empty.pending.is_empty());
        assert!(empty.done.is_empty());
        empty.move_data(example_todo(false));
        assert_eq!(todo.pending, empty.pending);
        assert_eq!(todo.done, empty.done);
    }

    #[test]
    fn version() {
        let mut todo = ToDo::default();
        assert_eq!(todo.get_version(), 0);
        todo.move_data(example_todo(false));
        assert_eq!(todo.get_version(), 1);
        todo.move_task(ToDoData::Done, 1);
    }

    #[test]
    fn toggle_filter() {
        let mut todo = example_todo(false);
        assert!(todo.project_filters.is_empty());
        todo.toggle_filter(ToDoCategory::Projects, "project1");
        assert!(todo.project_filters.contains("project1"));
        assert_eq!(todo.project_filters.len(), 1);
        todo.toggle_filter(ToDoCategory::Projects, "project1");
        assert!(todo.project_filters.is_empty());

        todo.toggle_filter(ToDoCategory::Contexts, "context1");
        assert!(todo.context_filters.contains("context1"));
        assert_eq!(todo.context_filters.len(), 1);
        todo.toggle_filter(ToDoCategory::Contexts, "context1");
        assert!(todo.context_filters.is_empty());

        todo.toggle_filter(ToDoCategory::Hashtags, "hashtag1");
        assert!(todo.hashtag_filters.contains("hashtag1"));
        assert_eq!(todo.hashtag_filters.len(), 1);
        todo.toggle_filter(ToDoCategory::Hashtags, "hashtag1");
        assert!(todo.hashtag_filters.is_empty());
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
        let mut todo = example_todo(false);
        todo.active = Some((ToDoData::Pending, 0));
        todo.update_active("New subject")?;
        assert_eq!(todo.pending[0].subject, "New subject");

        todo.active = Some((ToDoData::Done, 0));
        todo.update_active("New done subject")?;
        assert_eq!(todo.done[0].subject, "New done subject");

        Ok(())
    }

}
