use crate::CONFIG;
use std::convert::From;
use std::ops::Index;
use todo_txt::Task;
use tui::text::Span;
use tui::widgets::ListItem;

/// Represents a list of tasks, where each task is a tuple of `(usize, &'a Task)`.
/// The `usize` value is the index of the task in the original list.
pub struct TaskList<'a>(pub Vec<(usize, &'a Task)>);

impl<'a> TaskList<'a> {
    /// Returns the number of tasks in the list.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_actual_index(&self, index: usize) -> usize {
        self.0[index].0
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
            .map(|task| {
                let index = usize::from(u8::from(task.1.priority.clone()));
                let style = CONFIG.priority_colors.get_style(index);
                ListItem::new(Span::styled(task.1.subject.clone(), style))
            })
            .collect::<Vec<ListItem<'a>>>()
    }
}
