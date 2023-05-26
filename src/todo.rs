use std::collections::BTreeSet;
use std::error::Error;
use std::io::{BufRead, BufReader, Read};
use std::str::FromStr;
use todo_txt::Task;

#[allow(dead_code)]
struct ToDo {
    pending: Vec<Task>,
    done: Vec<Task>,
}

#[allow(dead_code)]
impl ToDo {
    pub fn load<R>(reader: R) -> Result<ToDo, Box<dyn Error>>
    where
        R: Read,
    {
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

        Ok(ToDo { pending, done })
    }

    fn get_btree(tasks: &Vec<Task>, f: fn(&Task) -> &Vec<String>) -> BTreeSet<String> {
        let mut btree = BTreeSet::new();
        tasks.iter().for_each(|task| {
            f(task).iter().for_each(|project| {
                btree.insert(project.clone());
            })
        });
        btree
    }

    pub fn get_projects(tasks: &Vec<Task>) -> BTreeSet<String> {
        Self::get_btree(tasks, |t| &t.projects)
    }

    pub fn get_contexts(tasks: &Vec<Task>) -> BTreeSet<String> {
        Self::get_btree(tasks, |t| &t.contexts)
    }

    pub fn get_hashtags(tasks: &Vec<Task>) -> BTreeSet<String> {
        Self::get_btree(tasks, |t| &t.hashtags)
    }

    fn get_tasks<'a>(
        tasks: &'a Vec<Task>,
        name: &str,
        f: fn(&Task) -> &Vec<String>,
    ) -> Vec<&'a Task> {
        tasks
            .iter()
            .filter(|task| f(task).contains(&String::from(name)))
            .collect()
    }

    fn get_tasks1<'a>(tasks: &'a Vec<Task>, name: &str) -> Vec<&'a Task> {
        tasks
            .iter()
            .filter(|task| task.projects.contains(&String::from(name)))
            .collect()
    }

    fn get_tasks2<'a>(tasks: &'a Vec<Task>, name: &str) -> Vec<&'a Task> {
        let mut ret: Vec<&'a Task> = Vec::new();
        let mut contains = false;
        for task in tasks {
            for project in &task.projects {
                if project == &String::from(name) {
                    contains = true;
                    break;
                }
            }
            if contains {
                ret.push(task);
                contains = false;
            }
        }
        ret
    }

    pub fn get_project_tasks<'a>(tasks: &'a Vec<Task>, name: &str) -> Vec<&'a Task> {
        Self::get_tasks(tasks, name, |t| &t.projects)
    }

    pub fn get_contexts_tasks<'a>(tasks: &'a Vec<Task>, name: &str) -> Vec<&'a Task> {
        Self::get_tasks(tasks, name, |t| &t.contexts)
    }

    fn get_hashtags_tasks<'a>(tasks: &'a Vec<Task>, name: &str) -> Vec<&'a Task> {
        Self::get_tasks(tasks, name, |t| &t.hashtags)
    }
}

#[cfg(test)]
mod tests {
    use super::ToDo;
    use std::error::Error;

    #[test]
    fn test_load() -> Result<(), Box<dyn Error>> {
        let todo = ToDo::load(
            r#"
        x (A) 2023-05-21 2023-04-30 measure space for +project1 @context1 #hashtag1 due:2023-06-30
                         2023-04-30 measure space for +project2 @context2           due:2023-06-30
          (C) 2023-04-30 measure space for +project3 @context3           due:2023-06-30
        "#.as_bytes(),
        )?;

        assert_eq!(todo.done.len(), 1);
        assert_eq!(todo.pending.len(), 2);

        assert_eq!(todo.done[0].priority, 0);
        assert!(todo.done[0].create_date.is_some());
        assert!(todo.done[0].finish_date.is_some());
        assert_eq!(todo.done[0].finished, true);
        assert_eq!(todo.done[0].threshold_date, None);
        assert!(todo.done[0].due_date.is_some());
        assert_eq!(todo.done[0].contexts.len(), 1);
        assert_eq!(todo.done[0].projects.len(), 1);
        assert_eq!(todo.done[0].hashtags.len(), 1);

        assert!(todo.pending[0].priority.is_lowest());
        assert!(todo.pending[0].create_date.is_some());
        assert!(todo.pending[0].finish_date.is_none());
        assert_eq!(todo.pending[0].finished, false);
        assert_eq!(todo.pending[0].threshold_date, None);
        assert!(todo.pending[0].due_date.is_some());
        assert_eq!(todo.pending[0].contexts.len(), 1);
        assert_eq!(todo.pending[0].projects.len(), 1);
        assert_eq!(todo.pending[0].hashtags.len(), 0);

        assert_eq!(todo.pending[1].priority, 2);
        assert!(todo.pending[1].create_date.is_some());
        assert!(todo.pending[1].finish_date.is_none());
        assert_eq!(todo.pending[1].finished, false);
        assert_eq!(todo.pending[1].threshold_date, None);
        assert!(todo.pending[1].due_date.is_some());
        assert_eq!(todo.pending[1].contexts.len(), 1);
        assert_eq!(todo.pending[1].projects.len(), 1);
        assert_eq!(todo.pending[1].hashtags.len(), 0);

        Ok(())
    }

    #[test]
    fn test_categeries_list() -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    #[test]
    fn test_tasks_in_categerie() -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
