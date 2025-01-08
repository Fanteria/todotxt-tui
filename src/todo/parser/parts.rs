use todo_txt::Task;

use super::{ToDo, ToDoData};

#[derive(Debug, PartialEq, Eq)]
pub enum Parts {
    Text(String),
    Pending,
    Done,
    Subject,
    Priority,
    CreateDate,
    FinishDate,
    Finished,
    TresholdDate,
    DueDate,
    Contexts,
    Projects,
    Hashtags,
    Special(String),
}

impl Parts {
    pub fn fill(&self, task: &Task, todo: &ToDo) -> Option<String> {
        use Parts::*;
        let process_vec = |vec: &[String]| {
            if vec.is_empty() {
                None
            } else {
                Some(vec.join(", "))
            }
        };
        match self {
            Text(text) => Some(text.to_string()),
            Pending => Some(todo.len(ToDoData::Pending).to_string()),
            Done => Some(todo.len(ToDoData::Done).to_string()),
            Subject => Some(task.subject.clone()),
            Priority => {
                if task.priority.is_lowest() {
                    None
                } else {
                    Some(task.priority.to_string())
                }
            }
            CreateDate => task.create_date.map(|d| d.to_string()),
            FinishDate => task.finish_date.map(|d| d.to_string()),
            Finished => Some(task.finished.to_string()),
            TresholdDate => task.threshold_date.map(|d| d.to_string()),
            DueDate => task.due_date.map(|d| d.to_string()),
            Contexts => process_vec(task.contexts()),
            Projects => process_vec(task.projects()),
            Hashtags => process_vec(&task.hashtags),
            Special(special) => task.tags.get(special).cloned(),
        }
    }
}

impl From<String> for Parts {
    fn from(value: String) -> Self {
        use Parts::*;
        match value.to_lowercase().as_str() {
            "pending" => Pending,
            "done" => Done,
            "subject" => Subject,
            "priority" => Priority,
            "create_date" => CreateDate,
            "finish_date" => FinishDate,
            "finished" => Finished,
            "treshold_date" => TresholdDate,
            "due_date" => DueDate,
            "contexts" => Contexts,
            "projects" => Projects,
            "hashtags" => Hashtags,
            _ => Special(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::error::Result;

    #[test]
    fn fill() -> Result<()> {
        let mut todo = ToDo::default();
        todo.new_task("task").unwrap();
        todo.new_task("(A) task").unwrap();
        todo.new_task("2023-11-12 task").unwrap();
        todo.new_task("task t:2023-11-12").unwrap();
        todo.new_task("task due:2023-11-12").unwrap();
        todo.new_task("task @context").unwrap();
        todo.new_task("task +project").unwrap();
        todo.new_task("task #hashtag").unwrap();
        todo.new_task("task spec:some-text").unwrap();
        todo.new_task("x 2023-11-12 2023-11-12 done task").unwrap();

        let task = Task::from_str("task").unwrap();
        assert_eq!(
            Parts::Text("Text".to_string()).fill(&task, &todo),
            Some(String::from("Text"))
        );
        assert_eq!(
            Parts::Text("Text".to_string()).fill(&task, &todo),
            Some(String::from("Text"))
        );
        assert_eq!(Parts::Pending.fill(&task, &todo), Some(String::from("9")));
        assert_eq!(Parts::Done.fill(&task, &todo), Some(String::from("1")));
        assert_eq!(
            Parts::Subject.fill(&task, &todo),
            Some(String::from("task"))
        );
        assert_eq!(Parts::Priority.fill(&task, &todo), None);

        let task = Task::from_str("(A) task").unwrap();
        assert_eq!(Parts::Priority.fill(&task, &todo), Some(String::from("A")));

        let task = Task::from_str("2023-11-12 task").unwrap();
        assert_eq!(
            Parts::CreateDate.fill(&task, &todo),
            Some(String::from("2023-11-12"))
        );
        assert_eq!(Parts::FinishDate.fill(&task, &todo), None);

        let task = Task::from_str("x 2023-11-12 2023-11-12 done task").unwrap();
        assert_eq!(
            Parts::FinishDate.fill(&task, &todo),
            Some(String::from("2023-11-12"))
        );
        assert_eq!(
            Parts::Finished.fill(&task, &todo),
            Some(String::from("true"))
        );
        assert_eq!(Parts::TresholdDate.fill(&task, &todo), None);

        let task = Task::from_str("task t:2023-11-12").unwrap();
        assert_eq!(
            Parts::TresholdDate.fill(&task, &todo),
            Some(String::from("2023-11-12"))
        );
        assert_eq!(Parts::DueDate.fill(&task, &todo), None);

        let task = Task::from_str("task due:2023-11-12").unwrap();
        assert_eq!(
            Parts::DueDate.fill(&task, &todo),
            Some(String::from("2023-11-12"))
        );
        assert_eq!(Parts::Contexts.fill(&task, &todo), None);

        let task = Task::from_str("task @context").unwrap();
        assert_eq!(
            Parts::Contexts.fill(&task, &todo),
            Some(String::from("context"))
        );
        assert_eq!(Parts::Projects.fill(&task, &todo), None);

        let task = Task::from_str("task +project").unwrap();
        assert_eq!(
            Parts::Projects.fill(&task, &todo),
            Some(String::from("project"))
        );
        assert_eq!(Parts::Hashtags.fill(&task, &todo), None);

        let task = Task::from_str("task #hashtag").unwrap();
        assert_eq!(
            Parts::Hashtags.fill(&task, &todo),
            Some(String::from("hashtag"))
        );
        assert_eq!(
            Parts::Special(String::from("spec")).fill(&task, &todo),
            None
        );

        let task = Task::from_str("task spec:some-text").unwrap();
        assert_eq!(
            Parts::Special(String::from("spec")).fill(&task, &todo),
            Some(String::from("some-text"))
        );

        Ok(())
    }
}
