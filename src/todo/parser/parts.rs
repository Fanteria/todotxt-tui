use super::ToDoData;
use super::ToDo;

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
                Priority => Some(task.priority.to_string()),
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
            "createDate" => CreateDate,
            "finishDate" => FinishDate,
            "finished" => Finished,
            "treshold_date" => TresholdDate,
            "dueDate" => DueDate,
            "contexts" => Contexts,
            "projects" => Projects,
            "hashtags" => Hashtags,
            _ => Special(value),
        }
    }
}
