use super::ToDo;
use super::ToDoData;

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
    pub fn fill(&self, todo: &ToDo) -> Option<String> {
        use Parts::*;
        let process_vec = |vec: &[String]| {
            if vec.is_empty() {
                None
            } else {
                Some(vec.join(", "))
            }
        };
        match todo.get_active() {
            Some(task) => match self {
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
            },
            None => None,
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
    use super::*;
    use crate::error::ToDoRes;

    #[test]
    fn fill() -> ToDoRes<()> {
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

        assert_eq!(
            Parts::Text("Text".to_string()).fill(&todo),
            None
        );

        todo.set_active(ToDoData::Pending, 0);
        assert_eq!(
            Parts::Text("Text".to_string()).fill(&todo),
            Some(String::from("Text"))
        );

        assert_eq!(
            Parts::Pending.fill(&todo),
            Some(String::from("9"))
        );

        assert_eq!(
            Parts::Done.fill(&todo),
            Some(String::from("1"))
        );

        assert_eq!(
            Parts::Subject.fill(&todo),
            Some(String::from("task"))
        );

        assert_eq!(
            Parts::Priority.fill(&todo),
            None
        );

        todo.set_active(ToDoData::Pending, 1);
        assert_eq!(
            Parts::Priority.fill(&todo),
            Some(String::from("A"))
        );

        todo.set_active(ToDoData::Pending, 2);
        assert_eq!(
            Parts::CreateDate.fill(&todo),
            Some(String::from("2023-11-12"))
        );

        assert_eq!(
            Parts::FinishDate.fill(&todo),
            None
        );

        todo.set_active(ToDoData::Done, 0);
        assert_eq!(
            Parts::FinishDate.fill(&todo),
            Some(String::from("2023-11-12"))
        );

        todo.set_active(ToDoData::Done, 0);
        assert_eq!(
            Parts::Finished.fill(&todo),
            Some(String::from("true"))
        );

        assert_eq!(
            Parts::TresholdDate.fill(&todo),
            None
        );

        todo.set_active(ToDoData::Pending, 3);
        assert_eq!(
            Parts::TresholdDate.fill(&todo),
            Some(String::from("2023-11-12"))
        );

        assert_eq!(
            Parts::DueDate.fill(&todo),
            None
        );

        todo.set_active(ToDoData::Pending, 4);
        assert_eq!(
            Parts::DueDate.fill(&todo),
            Some(String::from("2023-11-12"))
        );

        assert_eq!(
            Parts::Contexts.fill(&todo),
            None
        );

        todo.set_active(ToDoData::Pending, 5);
        assert_eq!(
            Parts::Contexts.fill(&todo),
            Some(String::from("context"))
        );

        assert_eq!(
            Parts::Projects.fill(&todo),
            None
        );

        todo.set_active(ToDoData::Pending, 6);
        assert_eq!(
            Parts::Projects.fill(&todo),
            Some(String::from("project"))
        );

        assert_eq!(
            Parts::Hashtags.fill(&todo),
            None
        );

        todo.set_active(ToDoData::Pending, 7);
        assert_eq!(
            Parts::Hashtags.fill(&todo),
            Some(String::from("hashtag"))
        );

        assert_eq!(
            Parts::Special(String::from("spec")).fill(&todo),
            None
        );

        todo.set_active(ToDoData::Pending, 8);
        assert_eq!(
            Parts::Special(String::from("spec")).fill(&todo),
            Some(String::from("some-text"))
        );

        Ok(())
    }
}
