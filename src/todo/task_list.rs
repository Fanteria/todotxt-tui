use crate::CONFIG;
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::ops::Index;
use todo_txt::Task;
use tui::text::Span;
use tui::widgets::ListItem;

type Item<'a> = (usize, &'a Task);

#[derive(Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum TaskSort {
    None,
    Reverse,
    Priority,
    Alphanumeric,
    AlphanumericReverse,
}

/// Represents a list of tasks, where each task is a tuple of `(usize, &'a Task)`.
/// The `usize` value is the index of the task in the original list.
pub struct TaskList<'a>(pub Vec<Item<'a>>);

pub struct TaskSlice<'a>(pub &'a [Item<'a>]);

impl<'a> TaskList<'a> {
    /// Returns the number of tasks in the list.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_actual_index(&self, index: usize) -> usize {
        self.0[index].0
    }

    pub fn slice(&self, first: usize, last: usize) -> TaskSlice {
        if last > self.0.len() {
            return TaskSlice(&self.0);
        };
        TaskSlice(&self.0[first..last])
    }

    pub fn sort(&mut self, sort: TaskSort) {
        use TaskSort::*;
        match sort {
            None => {}
            Reverse => self.0.reverse(),
            Priority => self
                .0
                .sort_by(|(_, a_task), (_, b_task)| b_task.priority.cmp(&a_task.priority)),
            Alphanumeric => self
                .0
                .sort_by(|(_, a_task), (_, b_task)| a_task.subject.cmp(&b_task.subject)),
            AlphanumericReverse => self
                .0
                .sort_by(|(_, a_task), (_, b_task)| b_task.subject.cmp(&a_task.subject)),
        }
    }
}

impl<'a> Index<usize> for TaskList<'a> {
    type Output = Task;
    fn index<'b>(&'b self, i: usize) -> &'a Task {
        self.0[i].1
    }
}

impl<'a> From<TaskList<'a>> for Vec<ListItem<'a>> {
    fn from(val: TaskList<'a>) -> Self {
        val.0
            .iter()
            .map(|(_, task)| {
                let index = usize::from(u8::from(task.priority.clone()));
                let style = CONFIG.priority_colors.get_style(index);
                ListItem::new(Span::styled(task.subject.clone(), style))
            })
            .collect::<Vec<ListItem<'a>>>()
    }
}

impl<'a> From<TaskSlice<'a>> for Vec<ListItem<'a>> {
    fn from(val: TaskSlice<'a>) -> Self {
        val.0
            .iter()
            .map(|(_, task)| {
                let index = usize::from(u8::from(task.priority.clone()));
                let style = CONFIG.priority_colors.get_style(index);
                ListItem::new(Span::styled(task.subject.clone(), style))
            })
            .collect::<Vec<ListItem<'a>>>()
    }
}
