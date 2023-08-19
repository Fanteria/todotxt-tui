pub mod category_list;
pub mod task_list;
pub use self::{category_list::CategoryList, task_list::TaskList};

use chrono::Utc;
use std::collections::btree_set::BTreeSet;
use std::convert::From;
use std::str::FromStr;
use todo_txt::Task;

/// Type alias for a tuple representing filter data.
type FilterData<'a> = (&'a BTreeSet<String>, fn(&'a Task) -> &'a [String]);

use ToDoData::Done;
use ToDoData::Pending;

#[derive(Clone, Copy)]
pub enum ToDoData {
    Pending,
    Done,
}

use ToDoCategory::Contexts;
use ToDoCategory::Hashtags;
use ToDoCategory::Projects;

#[derive(Clone, Copy)]
pub enum ToDoCategory {
    Projects,
    Contexts,
    Hashtags,
}

/// Represents a To-Do list.
#[derive(Default)]
pub struct ToDo {
    pub pending: Vec<Task>,
    pub done: Vec<Task>,
    use_done: bool,
    active: Option<(ToDoData, usize)>,
    pub project_filters: BTreeSet<String>,
    pub context_filters: BTreeSet<String>,
    pub hashtag_filters: BTreeSet<String>,
}

impl ToDo {
    /// Creates a new instance of `ToDo`.
    ///
    /// # Arguments
    ///
    /// * `use_done` - A boolean indicating whether to include
    ///         completed tasks in the list.
    ///
    /// # Returns
    ///
    /// A new instance of `ToDo`.
    pub fn new(use_done: bool) -> Self {
        Self {
            pending: Vec::new(),
            done: Vec::new(),
            use_done,
            active: None,
            project_filters: BTreeSet::new(),
            context_filters: BTreeSet::new(),
            hashtag_filters: BTreeSet::new(),
        }
    }

    pub fn move_data(&mut self, other: Self) {
        self.pending = other.pending;
        self.done = other.done;
    }

    pub fn get_data(&self, data: ToDoData) -> &Vec<Task> {
        match data {
            Pending => &self.pending,
            Done => &self.done,
        }
    }

    fn get_data_mut(&mut self, data: ToDoData) -> &mut Vec<Task> {
        match data {
            Pending => &mut self.pending,
            Done => &mut self.done,
        }
    }

    fn get_actual_index(&self, data: ToDoData, index: usize) -> usize {
        self.get_filtered(data).get_actual_index(index)
    }

    /// Adds a task to the To-Do list.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to add.
    pub fn add_task(&mut self, task: Task) {
        if task.finished {
            self.done.push(task);
        } else {
            self.pending.push(task);
        }
    }

    /// Constructs a [`CategoryList`] from a list of tasks, applying the
    /// provided filter function and checking for selected items.
    ///
    /// # Arguments
    ///
    /// * `tasks`: A vector of references to `Vec<Task>`
    ///         containing the tasks to be categorized.
    /// * `f`: A function that takes a reference to a `Task`
    ///         and returns a reference to a `Vec<String>`,
    ///         representing the category (e.g., projects,
    ///         contexts, hashtags).
    /// * `selected`: A reference to a `BTreeSet<String>`
    ///         containing the selected categories.
    ///
    /// # Returns
    ///
    /// A [`CategoryList`] containing the categorized items along with
    /// a boolean indicating whether each item is selected
    /// (contained in `selected`).
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

    /// Constructs a [`CategoryList`] from the current `ToDo` instance,
    /// applying the provided filter function and checking for selected
    /// items based on the [`ToDo::use_done`] flag.
    ///
    /// # Arguments
    ///
    /// * `f`: A function that takes a reference to a `Task`
    ///         and returns a reference to a `Vec<String>`,
    ///         representing the category (e.g., projects, contexts, hashtags).
    /// * `selected`: A reference to a `BTreeSet<String>` containing
    ///         the selected categories.
    ///
    /// # Returns
    ///
    /// A [`CategoryList`] containing the categorized items along
    /// with a boolean indicating whether each item is selected
    /// (contained in `selected`).
    fn get_btree_done_switch(
        &self,
        f: fn(&Task) -> &[String],
        selected: &BTreeSet<String>,
    ) -> CategoryList {
        Self::get_btree(
            if self.use_done {
                vec![&self.pending, &self.done]
            } else {
                vec![&self.pending]
            },
            f,
            selected,
        )
    }

    /// TODO rewrite comment
    /// Returns a [`CategoryList`] of all projects available
    /// in the current [`ToDo`] instance. If [`ToDo::use_done`] is enabled,
    /// it includes projects from both [`ToDo::pending`] and [`ToDo::done`] lists.
    ///
    /// # Returns
    ///
    /// A [`CategoryList`] of projects along with a boolean indicating whether
    /// each project is selected based on the applied filters.
    pub fn get_categories(&self, category: ToDoCategory) -> CategoryList {
        match category {
            Projects => self.get_btree_done_switch(|t| t.projects(), &self.project_filters),
            Contexts => self.get_btree_done_switch(|t| t.contexts(), &self.context_filters),
            Hashtags => self.get_btree_done_switch(|t| &t.hashtags, &self.hashtag_filters),
        }
    }

    /// Retrieves tasks that match a given name for a specific category
    /// (e.g., projects, contexts, hashtags).
    ///
    /// # Arguments
    ///
    /// * `tasks`: A vector of references to `Vec<Task>` containing
    ///         the tasks to be searched.
    /// * `name`: The name to search for within the category.
    /// * `f`: A function that takes a reference to a `Task` and returns a reference
    ///         to a `Vec<String>`, representing the category (e.g., projects, contexts, hashtags).
    ///
    /// # Returns
    ///
    /// A vector of references to the matching tasks.
    #[allow(dead_code)]
    fn get_tasks<'a>(
        tasks: Vec<&'a Vec<Task>>,
        name: &str,
        f: fn(&Task) -> &[String],
    ) -> Vec<&'a Task> {
        let mut vec = Vec::new();
        tasks.iter().for_each(|list| {
            vec.append(
                &mut list
                    .iter()
                    .filter(|task| f(task).contains(&String::from(name)))
                    .collect::<Vec<&'a Task>>(),
            );
        });
        vec
    }

    /// Moves a task from one list to another based on the provided index.
    ///
    /// # Arguments
    ///
    /// * `from`: A mutable reference to the source list of tasks.
    /// * `to`: A mutable reference to the destination list of tasks.
    /// * `index`: The index of the task to be moved from the source list
    ///         to the destination list.
    fn move_task_logic(from: &mut Vec<Task>, to: &mut Vec<Task>, index: usize) {
        if from.len() <= index {
            return;
        }
        to.push(from.remove(index))
    }

    pub fn move_task(&mut self, data: ToDoData, index: usize) {
        let index = self.get_actual_index(data, index);
        match data {
            Pending => Self::move_task_logic(&mut self.pending, &mut self.done, index),
            Done => Self::move_task_logic(&mut self.done, &mut self.pending, index),
        };
    }

    /// Retrieves tasks that match a given name for a specific category
    /// (e.g., projects, contexts, hashtags) based on the [`ToDo::use_done`] flag.
    ///
    /// # Arguments
    ///
    /// * `name`: The name to search for within the category.
    /// * `f`: A function that takes a reference to a `Task` and returns
    ///         a reference to a `Vec<String>`,
    ///         representing the category (e.g., projects, contexts, hashtags).
    ///
    /// # Returns
    ///
    /// A vector of references to the matching tasks.
    #[allow(dead_code)]
    fn get_tasks_done_switch<'a>(&'a self, name: &str, f: fn(&Task) -> &[String]) -> Vec<&'a Task> {
        Self::get_tasks(
            if self.use_done {
                vec![&self.pending, &self.done]
            } else {
                vec![&self.pending]
            },
            name,
            f,
        )
    }

    /// Filters tasks based on the provided filters and returns a `TaskList`.
    ///
    /// # Arguments
    ///
    /// * `tasks`: A reference to a slice of `Task` instances to be filtered.
    /// * `filters`: A reference to a slice of [`FilterData`] tuples containing
    ///             filter categories and functions. Each [`FilterData`] tuple
    ///             consists of a reference to a `BTreeSet<String>` representing
    ///             the selected categories and a function that takes a reference
    ///             to a `Task` and returns a reference to a `Vec<String>`,
    ///             representing the category to be filtered (e.g., projects,
    ///             contexts, hashtags).
    ///
    /// # Returns
    ///
    /// A `TaskList` containing the filtered tasks along with their respective indices.
    fn get_filtered_tasks<'a>(tasks: &'a [Task], filters: &[FilterData<'a>]) -> TaskList<'a> {
        TaskList(
            tasks
                .iter()
                .enumerate()
                .filter(|task| {
                    filters
                        .iter()
                        .all(|filter| filter.0.iter().all(|item| filter.1(task.1).contains(item)))
                })
                .collect(),
        )
    }

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

    pub fn get_filtered(&self, data: ToDoData) -> TaskList {
        Self::get_filtered_tasks(
            self.get_data(data),
            &[
                (&self.project_filters, |t| t.projects()),
                (&self.context_filters, |t| t.contexts()),
                (&self.hashtag_filters, |t| &t.hashtags),
            ],
        )
    }

    #[allow(dead_code)]
    pub fn get_all(&self, data: ToDoData) -> TaskList {
        TaskList(self.get_data(data).iter().enumerate().collect())
    }

    /// Adds a new task to the `ToDo` instance.
    ///
    /// # Arguments
    ///
    /// * `task`: The new task to be added as a string.
    ///
    /// # Returns
    ///
    /// An `Ok` result if the task was successfully added, or an `Err` containing
    /// a `todo_txt::Error` if there was a problem parsing the task.
    pub fn new_task(&mut self, task: &str) -> Result<(), todo_txt::Error> {
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

    pub fn remove_task(&mut self, data: ToDoData, index: usize) {
        let index = self.get_actual_index(data, index);
        self.get_data_mut(data).remove(index);
    }

    pub fn swap_tasks(&mut self, data: ToDoData, from: usize, to: usize) {
        let from = self.get_actual_index(data, from);
        let to = self.get_actual_index(data, to);
        self.get_data_mut(data).swap(from, to);
        // TODO index out of range
    }

    pub fn set_active(&mut self, data: ToDoData, index: usize) {
        let index = self.get_actual_index(data, index);
        self.active = Some((data, index));
    }

    pub fn get_active(&self) -> Option<&Task> {
        match self.active {
            Some((data, index)) => Some(&self.get_data(data)[index]),
            None => None,
        }
    }

    pub fn update_active(&mut self, task: &str) -> Result<(), todo_txt::Error> {
        if let Some((data, index)) = self.active {
            self.get_data_mut(data)[index] = Task::from_str(task)?;
        }
        Ok(())
    }

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
}
