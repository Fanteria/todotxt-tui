pub mod category_list;
pub mod task_list;
pub use self::{category_list::CategoryList, task_list::TaskList};

use std::collections::btree_set::BTreeSet;
use std::convert::From;
use std::str::FromStr;
use todo_txt::Task;

type FilterData<'a> = (&'a BTreeSet<String>, fn(&'a Task) -> &'a Vec<String>);

#[derive(Default)]
pub struct ToDo {
    pub pending: Vec<Task>,
    pub done: Vec<Task>,
    use_done: bool,
    pub project_filters: BTreeSet<String>,
    pub context_filters: BTreeSet<String>,
    pub hashtag_filters: BTreeSet<String>,
}

impl ToDo {
    pub fn new(use_done: bool) -> Self {
        Self {
            pending: Vec::new(),
            done: Vec::new(),
            use_done,
            project_filters: BTreeSet::new(),
            context_filters: BTreeSet::new(),
            hashtag_filters: BTreeSet::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        if task.finished {
            self.done.push(task);
        } else {
            self.pending.push(task);
        }
    }

    fn get_btree<'a>(
        tasks: Vec<&'a Vec<Task>>,
        f: fn(&Task) -> &Vec<String>,
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

    fn get_btree_done_switch(
        &self,
        f: fn(&Task) -> &Vec<String>,
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

    pub fn get_projects(&self) -> CategoryList {
        self.get_btree_done_switch(|t| &t.projects, &self.project_filters)
    }

    pub fn get_contexts(&self) -> CategoryList {
        self.get_btree_done_switch(|t| &t.contexts, &self.context_filters)
    }

    pub fn get_hashtags(&self) -> CategoryList {
        self.get_btree_done_switch(|t| &t.hashtags, &self.hashtag_filters)
    }

    fn get_tasks<'a>(
        tasks: Vec<&'a Vec<Task>>,
        name: &str,
        f: fn(&Task) -> &Vec<String>,
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

    fn move_task(from: &mut Vec<Task>, to: &mut Vec<Task>, index: usize) {
        if from.len() <= index {
            return;
        }
        to.push(from.remove(index))
    }

    pub fn move_pending_task(&mut self, index: usize) {
        Self::move_task(&mut self.pending, &mut self.done, index)
    }

    pub fn move_done_task(&mut self, index: usize) {
        Self::move_task(&mut self.done, &mut self.pending, index)
    }

    fn get_tasks_done_switch<'a>(
        &'a self,
        name: &str,
        f: fn(&Task) -> &Vec<String>,
    ) -> Vec<&'a Task> {
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

    pub fn get_project_tasks<'a>(&'a self, name: &str) -> Vec<&'a Task> {
        self.get_tasks_done_switch(name, |t| &t.projects)
    }

    pub fn get_context_tasks<'a>(&'a self, name: &str) -> Vec<&'a Task> {
        self.get_tasks_done_switch(name, |t| &t.contexts)
    }

    pub fn get_hashtag_tasks<'a>(&'a self, name: &str) -> Vec<&'a Task> {
        self.get_tasks_done_switch(name, |t| &t.hashtags)
    }

    fn get_filtered<'a>(tasks: &'a [Task], filters: &[FilterData<'a>]) -> TaskList<'a> {
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

    pub fn toggle_filter(filter_set: &mut BTreeSet<String>, filter: &str) {
        let filter = String::from(filter);
        if !filter_set.insert(filter.clone()) {
            filter_set.remove(&filter);
        }
    }

    pub fn get_pending_filtered(&self) -> TaskList {
        Self::get_filtered(
            &self.pending,
            &[
                (&self.project_filters, |t| &t.projects),
                (&self.context_filters, |t| &t.contexts),
                (&self.hashtag_filters, |t| &t.hashtags),
            ],
        )
    }

    pub fn get_pending_all(&self) -> TaskList {
        TaskList(self.pending.iter().enumerate().collect())
    }

    pub fn get_done_filtered(&self) -> TaskList {
        Self::get_filtered(
            &self.done,
            &[
                (&self.project_filters, |t| &t.projects),
                (&self.context_filters, |t| &t.contexts),
                (&self.hashtag_filters, |t| &t.hashtags),
            ],
        )
    }

    pub fn get_done_all(&self) -> TaskList {
        TaskList(self.done.iter().enumerate().collect())
    }

    pub fn new_task(&mut self, task: &str) -> Result<(), todo_txt::Error> {
        let task = Task::from_str(task)?;
        if task.finished {
            self.done.push(task);
        } else {
            self.pending.push(task);
        }
        Ok(())
    }

    pub fn remove_pending_task(&mut self, index: usize) {
        self.pending.remove(index);
    }

    pub fn finish_task(&mut self, index: usize) {
        self.done.push(self.pending.remove(index));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::naive::NaiveDate;
    use std::error::Error;
    use todo_txt::Priority;

    fn task_pch(
        subject: &str,
        projects: Vec<&str>,
        contexts: Vec<&str>,
        hashtags: Vec<&str>,
    ) -> Task {
        let mut task = task_pc(subject, projects, contexts);
        if !hashtags.is_empty() {
            task.subject += " #";
            task.subject += &hashtags.join(" #");
            task.hashtags = hashtags.iter().map(|h| String::from(*h)).collect();
        }
        task
    }

    fn task_pc(subject: &str, projects: Vec<&str>, contexts: Vec<&str>) -> Task {
        let mut task = task_p(subject, projects);
        if !contexts.is_empty() {
            task.subject += " @";
            task.subject += &contexts.join(" @");
            task.contexts = contexts.iter().map(|c| String::from(*c)).collect();
        }
        task
    }

    fn task_p(subject: &str, projects: Vec<&str>) -> Task {
        let mut task = Task::default();
        task.subject = String::from(subject);
        if !projects.is_empty() {
            task.subject += " +";
            task.subject += &projects.join(" +");
            task.projects = projects.iter().map(|p| String::from(*p)).collect();
        }
        task
    }

    fn example_todo(use_done: bool) -> ToDo {
        let mut todo = ToDo::new(use_done);

        let mut task = task_pch(
            "measure space for 1",
            vec!["project1"],
            vec!["context1"],
            vec!["hashtag1"],
        );
        task.finished = true;
        task.priority = Priority::from(0);
        task.create_date = Some(NaiveDate::from_ymd_opt(2023, 4, 30).unwrap());
        task.finish_date = Some(NaiveDate::from_ymd_opt(2023, 5, 21).unwrap());
        task.due_date = Some(NaiveDate::from_ymd_opt(2023, 6, 30).unwrap());
        todo.add_task(task);

        let mut task = task_pch(
            "measure space for 2",
            vec!["project2"],
            vec!["context2"],
            vec![],
        );
        task.create_date = Some(NaiveDate::from_ymd_opt(2023, 4, 30).unwrap());
        task.due_date = Some(NaiveDate::from_ymd_opt(2023, 6, 30).unwrap());
        todo.add_task(task);

        let mut task = task_pch(
            "measure space for 3",
            vec!["project3"],
            vec!["context3"],
            vec![],
        );
        task.priority = Priority::from(2);
        task.create_date = Some(NaiveDate::from_ymd_opt(2023, 4, 30).unwrap());
        task.due_date = Some(NaiveDate::from_ymd_opt(2023, 6, 30).unwrap());
        todo.add_task(task);

        let mut task = task_pch(
            "measure space for",
            vec!["project2"],
            vec!["context3"],
            vec!["hashtag1"],
        );
        task.due_date = Some(NaiveDate::from_ymd_opt(2023, 6, 30).unwrap());
        todo.add_task(task);

        let mut task = task_pch(
            "measure space for 5",
            vec!["project3"],
            vec!["context3"],
            vec!["hashtag2"],
        );
        task.finished = true;
        task.due_date = Some(NaiveDate::from_ymd_opt(2023, 6, 30).unwrap());
        todo.add_task(task);

        let mut task = task_pch(
            "measure space for 6",
            vec!["project3"],
            vec!["context2"],
            vec!["hashtag2"],
        );
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
        assert_eq!(todo.done[0].contexts.len(), 1);
        assert_eq!(todo.done[0].projects.len(), 1);
        assert_eq!(todo.done[0].hashtags.len(), 1);

        println!("{:#?}", todo.pending[0]);

        assert!(todo.pending[0].priority.is_lowest());
        assert!(todo.pending[0].create_date.is_some());
        assert!(todo.pending[0].finish_date.is_none());
        assert!(!todo.pending[0].finished);
        assert_eq!(todo.pending[0].threshold_date, None);
        assert!(todo.pending[0].due_date.is_some());
        assert_eq!(todo.pending[0].contexts.len(), 1);
        assert_eq!(todo.pending[0].projects.len(), 1);
        assert_eq!(todo.pending[0].hashtags.len(), 0);

        assert_eq!(todo.pending[1].priority, 2);
        assert!(todo.pending[1].create_date.is_some());
        assert!(todo.pending[1].finish_date.is_none());
        assert!(!todo.pending[1].finished);
        assert_eq!(todo.pending[1].threshold_date, None);
        assert!(todo.pending[1].due_date.is_some());
        assert_eq!(todo.pending[1].contexts.len(), 1);
        assert_eq!(todo.pending[1].projects.len(), 1);
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
            todo.get_projects().0,
            create_vec(&[String::from("project2"), String::from("project3")])
        );
        assert_eq!(
            todo.get_contexts().0,
            create_vec(&[String::from("context2"), String::from("context3")])
        );
        assert_eq!(
            todo.get_hashtags().0,
            create_vec(&[String::from("hashtag1"), String::from("hashtag2")])
        );

        todo.use_done = true;
        assert_eq!(
            todo.get_projects().0,
            create_vec(&[
                String::from("project1"),
                String::from("project2"),
                String::from("project3"),
            ])
        );
        assert_eq!(
            todo.get_contexts().0,
            create_vec(&[
                String::from("context1"),
                String::from("context2"),
                String::from("context3"),
            ])
        );
        assert_eq!(
            todo.get_hashtags().0,
            create_vec(&[String::from("hashtag1"), String::from("hashtag2")])
        );

        Ok(())
    }

    #[test]
    fn test_tasks_in_category() -> Result<(), Box<dyn Error>> {
        let mut todo = example_todo(false);
        assert_eq!(todo.get_project_tasks("project1").len(), 0);
        assert_eq!(todo.get_project_tasks("project2").len(), 2);
        assert_eq!(todo.get_project_tasks("project3").len(), 2);
        assert_eq!(todo.get_context_tasks("context1").len(), 0);
        assert_eq!(todo.get_context_tasks("context2").len(), 2);
        assert_eq!(todo.get_context_tasks("context3").len(), 2);
        assert_eq!(todo.get_hashtag_tasks("hashtag1").len(), 1);
        assert_eq!(todo.get_hashtag_tasks("hashtag2").len(), 1);

        todo.use_done = true;
        assert_eq!(todo.get_project_tasks("project1").len(), 1);
        assert_eq!(todo.get_project_tasks("project2").len(), 2);
        assert_eq!(todo.get_project_tasks("project3").len(), 3);
        assert_eq!(todo.get_context_tasks("context1").len(), 1);
        assert_eq!(todo.get_context_tasks("context2").len(), 2);
        assert_eq!(todo.get_context_tasks("context3").len(), 3);
        assert_eq!(todo.get_hashtag_tasks("hashtag1").len(), 2);
        assert_eq!(todo.get_hashtag_tasks("hashtag2").len(), 2);

        Ok(())
    }

    #[test]
    fn test_filtering() -> Result<(), Box<dyn Error>> {
        let mut todo = ToDo::new(false);
        todo.add_task(task_p("task 1", vec![]));
        todo.add_task(task_p("task 2", vec!["project1"]));
        todo.add_task(task_p("task 3", vec!["project1", "project2"]));
        todo.add_task(task_p("task 4", vec!["project1", "project3"]));
        todo.add_task(task_p("task 5", vec!["project1", "project2", "project3"]));
        todo.add_task(task_pch(
            "task 6",
            vec!["project3"],
            vec!["context2"],
            vec!["hashtag2", "hashtag1"],
        ));
        todo.add_task(task_pch(
            "task 7",
            vec!["project2"],
            vec!["context1"],
            vec!["hashtag1", "hashtag2"],
        ));
        todo.add_task(task_pc("task 8", vec!["project2"], vec!["context2"]));
        todo.add_task(task_pc("task 9", vec!["projects3"], vec!["context3"]));
        todo.add_task(task_pch(
            "task 10",
            vec!["project2"],
            vec!["context3"],
            vec!["hashtag1", "hashtag2"],
        ));
        todo.add_task(task_pch(
            "task 11",
            vec!["project3"],
            vec!["context3"],
            vec!["hashtag2", "hashtag3"],
        ));
        todo.add_task(task_pch(
            "task 12",
            vec!["project3"],
            vec!["context2"],
            vec!["hashtag2"],
        ));

        let filtered = todo.get_pending_filtered();
        assert_eq!(filtered.len(), 12);

        todo.project_filters.insert(String::from("project9999"));
        let filtered = todo.get_pending_filtered();
        assert_eq!(filtered.len(), 0);

        todo.project_filters.clear();
        todo.project_filters.insert(String::from("project1"));
        let filtered = todo.get_pending_filtered();
        assert_eq!(filtered.len(), 4);
        assert_eq!(filtered[0].subject, "task 2 +project1");
        assert_eq!(filtered[1].subject, "task 3 +project1 +project2");
        assert_eq!(filtered[2].subject, "task 4 +project1 +project3");
        assert_eq!(filtered[3].subject, "task 5 +project1 +project2 +project3");

        todo.project_filters.insert(String::from("project2"));
        let filtered = todo.get_pending_filtered();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].subject, "task 3 +project1 +project2");
        assert_eq!(filtered[1].subject, "task 5 +project1 +project2 +project3");

        todo.project_filters.insert(String::from("project3"));
        let filtered = todo.get_pending_filtered();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].subject, "task 5 +project1 +project2 +project3");

        todo.project_filters.insert(String::from("project1"));
        let filtered = todo.get_pending_filtered();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].subject, "task 5 +project1 +project2 +project3");

        todo.project_filters.clear();
        todo.context_filters.insert(String::from("context1"));
        let filtered = todo.get_pending_filtered();
        assert_eq!(filtered.len(), 1);
        assert_eq!(
            filtered[0].subject,
            "task 7 +project2 @context1 #hashtag1 #hashtag2"
        );

        Ok(())
    }
}
