use std::collections::btree_set::BTreeSet;
use std::convert::From;
use std::error::Error;
use std::io::{BufRead, BufReader, Read, Result as ioResult, Write};
use std::str::FromStr;
use todo_txt::Task;
use tui::text::Span;
use tui::widgets::ListItem;

pub struct ToDo {
    pub pending: TaskList,
    pub done: TaskList,
    use_done: bool,
    // stack:
}

impl ToDo {
    pub fn new(use_done: bool) -> Self {
        Self {
            pending: TaskList(Vec::new()),
            done: TaskList(Vec::new()),
            use_done,
        }
    }

    pub fn load<R: Read>(reader: R, use_done: bool) -> Result<ToDo, Box<dyn Error>> {
        let mut pending = Vec::new();
        let mut done = Vec::new();
        for line in BufReader::new(reader).lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let task = Task::from_str(&line)?;
            if task.finished {
                done.push(task);
            } else {
                pending.push(task);
            }
        }

        Ok(ToDo {
            pending: TaskList(pending),
            done: TaskList(done),
            use_done,
        })
    }

    fn get_btree(tasks: Vec<&TaskList>, f: fn(&Task) -> &Vec<String>) -> CategoryList {
        let mut btree = BTreeSet::new();

        tasks.iter().for_each(|list| {
            list.0.iter().for_each(|task| {
                f(task).iter().for_each(|project| {
                    btree.insert(project.clone());
                })
            })
        });
        CategoryList(btree)
    }

    fn get_btree_done_switch(&self, f: fn(&Task) -> &Vec<String>) -> CategoryList {
        Self::get_btree(
            if self.use_done {
                vec![&self.pending, &self.done]
            } else {
                vec![&self.pending]
            },
            f,
        )
    }

    pub fn get_projects(&self) -> CategoryList {
        self.get_btree_done_switch(|t| &t.projects)
    }

    pub fn get_contexts(&self) -> CategoryList {
        self.get_btree_done_switch(|t| &t.contexts)
    }

    pub fn get_hashtags(&self) -> CategoryList {
        self.get_btree_done_switch(|t| &t.hashtags)
    }

    fn get_tasks<'a>(
        tasks: Vec<&'a TaskList>,
        name: &str,
        f: fn(&Task) -> &Vec<String>,
    ) -> Vec<&'a Task> {
        let mut vec = Vec::new();
        tasks.iter().for_each(|list| {
            vec.append(
                &mut list
                    .0
                    .iter()
                    .filter(|task| f(task).contains(&String::from(name)))
                    .collect::<Vec<&'a Task>>(),
            );
        });
        vec
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

    fn write_tasks<W: Write>(tasks: &TaskList, writer: &mut W) -> ioResult<()> {
        for task in &tasks.0 {
            writer.write((task.to_string() + "\n").as_bytes())?;
        }
        Ok(())
    }

    pub fn write_done_tasks<W: Write>(&self, writer: &mut W) -> ioResult<()> {
        Self::write_tasks(&self.done, writer)
    }

    pub fn write_pending_tasks<W: Write>(&self, writer: &mut W) -> ioResult<()> {
        Self::write_tasks(&self.pending, writer)
    }

    pub fn write_all_tasks<W: Write>(&self, writer: &mut W) -> ioResult<()> {
        self.write_done_tasks(writer)?;
        self.write_pending_tasks(writer)
    }

    pub fn get_pending_tasks(&self) -> &TaskList {
        &self.pending
    }

    pub fn new_task(&mut self, task: &str) -> Result<(), todo_txt::Error> {
        let task = Task::from_str(task)?;
        if task.finished {
            self.done.0.push(task);
        } else {
            self.pending.0.push(task);
        }
        Ok(())
    }

    pub fn remove_pending_task(&mut self, index: usize) {
        self.pending.remove_task(index);
    }

    pub fn finish_task(&mut self, index: usize) {
        self.done.add_task(self.pending.remove_task(index));
    }
}

#[derive(Clone)]
pub struct TaskList(pub Vec<Task>);

impl<'a> Into<Vec<ListItem<'a>>> for TaskList {
    fn into(self) -> Vec<ListItem<'a>> {
        self.0
            .iter()
            .map(|task| ListItem::new(task.subject.clone()))
            .collect::<Vec<ListItem<'a>>>()
    }
}

impl TaskList {
    pub fn remove_task(&mut self, index: usize) -> Task {
        self.0.remove(index)
    }

    pub fn add_task(&mut self, task: Task) {
        self.0.push(task);
    }
}

pub struct CategoryList(pub BTreeSet<String>);

impl CategoryList {
    pub fn start_with(&self, pattern: &str) -> Vec<&String> {
        self.0
            .iter()
            .filter(|item| item.starts_with(pattern))
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> Into<Vec<ListItem<'a>>> for CategoryList {
    fn into(self) -> Vec<ListItem<'a>> {
        self.0
            .iter()
            .map(|category| {
                if category == "project2" {
                    ListItem::new(Span::styled(
                        category.clone(),
                        tui::style::Style::default().fg(tui::style::Color::Blue),
                    ))
                } else {
                    ListItem::new(category.clone())
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TESTING_STRING: &str = r#"
        x (A) 2023-05-21 2023-04-30 measure space for 1 +project1 @context1 #hashtag1 due:2023-06-30
                         2023-04-30 measure space for 2 +project2 @context2           due:2023-06-30
                     (C) 2023-04-30 measure space for 3 +project3 @context3           due:2023-06-30
                                    measure space for 4 +project2 @context3 #hashtag1 due:2023-06-30
                                  x measure space for 5 +project3 @context3 #hashtag2 due:2023-06-30
                                    measure space for 6 +project3 @context2 #hashtag2 due:2023-06-30
        "#;

    #[test]
    fn test_load() -> Result<(), Box<dyn Error>> {
        let todo = ToDo::load(TESTING_STRING.as_bytes(), true)?;

        assert_eq!(todo.done.0.len(), 2);
        assert_eq!(todo.pending.0.len(), 4);

        assert_eq!(todo.done.0[0].priority, 0);
        assert!(todo.done.0[0].create_date.is_some());
        assert!(todo.done.0[0].finish_date.is_some());
        assert_eq!(todo.done.0[0].finished, true);
        assert_eq!(todo.done.0[0].threshold_date, None);
        assert!(todo.done.0[0].due_date.is_some());
        assert_eq!(todo.done.0[0].contexts.len(), 1);
        assert_eq!(todo.done.0[0].projects.len(), 1);
        assert_eq!(todo.done.0[0].hashtags.len(), 1);

        assert!(todo.pending.0[0].priority.is_lowest());
        assert!(todo.pending.0[0].create_date.is_some());
        assert!(todo.pending.0[0].finish_date.is_none());
        assert_eq!(todo.pending.0[0].finished, false);
        assert_eq!(todo.pending.0[0].threshold_date, None);
        assert!(todo.pending.0[0].due_date.is_some());
        assert_eq!(todo.pending.0[0].contexts.len(), 1);
        assert_eq!(todo.pending.0[0].projects.len(), 1);
        assert_eq!(todo.pending.0[0].hashtags.len(), 0);

        assert_eq!(todo.pending.0[1].priority, 2);
        assert!(todo.pending.0[1].create_date.is_some());
        assert!(todo.pending.0[1].finish_date.is_none());
        assert_eq!(todo.pending.0[1].finished, false);
        assert_eq!(todo.pending.0[1].threshold_date, None);
        assert!(todo.pending.0[1].due_date.is_some());
        assert_eq!(todo.pending.0[1].contexts.len(), 1);
        assert_eq!(todo.pending.0[1].projects.len(), 1);
        assert_eq!(todo.pending.0[1].hashtags.len(), 0);

        Ok(())
    }

    #[test]
    fn test_categeries_list() -> Result<(), Box<dyn Error>> {
        let create_btree = |items: &[&str]| {
            let mut btree: BTreeSet<String> = BTreeSet::new();
            items.iter().for_each(|item| {
                btree.insert(item.to_string());
            });
            btree
        };

        let mut todo = ToDo::load(TESTING_STRING.as_bytes(), false)?;
        assert_eq!(
            todo.get_projects().0,
            create_btree(&["project2", "project3"])
        );
        assert_eq!(
            todo.get_contexts().0,
            create_btree(&["context2", "context3"])
        );
        assert_eq!(
            todo.get_hashtags().0,
            create_btree(&["hashtag1", "hashtag2"])
        );

        todo.use_done = true;
        assert_eq!(
            todo.get_projects().0,
            create_btree(&["project1", "project2", "project3"])
        );
        assert_eq!(
            todo.get_contexts().0,
            create_btree(&["context1", "context2", "context3"])
        );
        assert_eq!(
            todo.get_hashtags().0,
            create_btree(&["hashtag1", "hashtag2"])
        );

        Ok(())
    }

    #[test]
    fn test_tasks_in_category() -> Result<(), Box<dyn Error>> {
        let mut todo = ToDo::load(TESTING_STRING.as_bytes(), false)?;
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
    fn test_write_tasks() -> Result<(), Box<dyn Error>> {
        let todo = ToDo::load(TESTING_STRING.as_bytes(), false)?;
        let mut buf: Vec<u8> = Vec::new();

        let mut test_function =
            |function: fn(&ToDo, &mut Vec<_>) -> ioResult<()>, f: fn(&String) -> bool, message| {
                // run function
                function(&todo, &mut buf).unwrap();
                // get testing data
                let expected = TESTING_STRING
                    .trim()
                    .lines()
                    .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
                    .filter(|line| f(line))
                    .collect::<Vec<String>>()
                    .join("\n")
                    + "\n";
                assert_eq!(
                    expected.as_bytes(),
                    buf,
                    // if test failed print data in string not in byte array
                    "\n-----{}-----\nGET:\n{}\n----------------\nEXPECTED:\n{}\n",
                    message,
                    String::from_utf8(buf.clone()).unwrap(),
                    expected.clone()
                );
                buf.clear();
            };

        test_function(
            ToDo::write_done_tasks,
            |f| f.starts_with("x "),
            "Pending check is wrong",
        );

        test_function(
            ToDo::write_pending_tasks,
            |f| !f.starts_with("x "),
            "Pending check is wrong",
        );

        Ok(())
    }
}
