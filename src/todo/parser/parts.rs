use std::str::FromStr;

use super::{Rule, ToDo, ToDoData};
use crate::{
    config::{Color, Styles, TextModifier, TextStyle},
    ToDoError,
};
use pest::iterators::Pairs;
use todo_txt::{Priority as TaskPriority, Task};

#[derive(Default, Clone, Copy, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum PartStyleValue {
    Const(TextStyle),
    #[default]
    Priority,
    SpecificPriority(u8),
    CustomCategory,
    Projects,
    Contexts,
    Hashtags,
    Category,
}

impl PartStyleValue {
    fn get_style(&self, task: &Task, styles: &Styles) -> TextStyle {
        match self {
            Self::Const(style) => *style,
            Self::CustomCategory => {
                let mut text_style = TextStyle::default();
                let mut process_projects = |prefix: &str, data: &[String]| {
                    data.iter().for_each(|category: &String| {
                        if let Some(style) = styles
                            .custom_category_style
                            .get(&(prefix.to_string() + category))
                        {
                            text_style = text_style.combine(style);
                        }
                    });
                };
                process_projects("+", task.projects());
                process_projects("@", task.contexts());
                process_projects("#", &task.hashtags);

                text_style
            }
            Self::Priority => {
                let priority = if task.priority.is_lowest() && task.finished {
                    // For completed tasks, get priority from pri: tag
                    task.tags.get("pri")
                        .and_then(|p| todo_txt::Priority::try_from(p.chars().next().unwrap_or('Z')).ok())
                        .unwrap_or_else(|| task.priority.clone())
                } else {
                    task.priority.clone()
                };
                styles.priority_style.get_text_style(priority.into())
            }
            Self::SpecificPriority(p) => styles.priority_style.get_text_style(*p),
            Self::Projects => styles.projects_style,
            Self::Contexts => styles.contexts_style,
            Self::Hashtags => styles.hashtags_style,
            Self::Category => styles.category_style,
        }
    }
}

#[derive(Default, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct PartStyle {
    pub style: Vec<PartStyleValue>,
    pub to_colorize: bool,
    pub skip_projects: bool,
    pub skip_contexts: bool,
    pub skip_hashtags: bool,
}

impl PartStyle {
    pub fn get_style(&self, task: &Task, styles: &Styles) -> TextStyle {
        self.style
            .iter()
            .fold(TextStyle::default(), |accumulate, right| {
                accumulate.combine(&right.get_style(task, styles))
            })
    }
}

impl TryFrom<Pairs<'_, Rule>> for PartStyle {
    type Error = ToDoError;

    fn try_from(pairs: Pairs<'_, Rule>) -> Result<Self, Self::Error> {
        let mut s = Self::default();
        for value in pairs {
            use PartStyleValue::*;
            match value.as_rule() {
                Rule::bang => s.to_colorize = true,
                Rule::style_specific_priority => s.style.push(SpecificPriority(
                    TaskPriority::try_from(value.as_str().chars().last().unwrap())?.into(),
                )),
                Rule::style_priority => s.style.push(Priority),
                Rule::style_custom_category => s.style.push(CustomCategory),
                Rule::style_projects => s.style.push(Projects),
                Rule::style_contexts => s.style.push(Contexts),
                Rule::style_hashtags => s.style.push(Hashtags),
                Rule::style_category => s.style.push(Category),
                Rule::style_color => {
                    let mut it = value.into_inner().rev();
                    let name = it.next().unwrap().as_str();
                    if let Ok(color) = Color::from_str(name) {
                        s.style.push(Const(if it.next().is_some() {
                            TextStyle::default().bg(color)
                        } else {
                            TextStyle::default().fg(color)
                        }))
                    } else {
                        s.style.push(Const(
                            TextStyle::default().modifier(TextModifier::from_str(name)?),
                        ));
                    }
                }
                Rule::style_skip_projects => s.skip_projects = true,
                Rule::style_skip_contexts => s.skip_contexts = true,
                Rule::style_skip_hashtags => s.skip_hashtags = true,
                _ => unreachable!(),
            }
        }
        Ok(s)
    }
}

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
                    // For completed tasks, check pri: tag
                    if task.finished {
                        task.tags.get("pri").cloned()
                    } else {
                        None
                    }
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

        let task = Task::from_str("x task pri:B").unwrap();
        assert_eq!(Parts::Priority.fill(&task, &todo), Some(String::from("B")));

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
