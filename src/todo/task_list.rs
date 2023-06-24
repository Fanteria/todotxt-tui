use crate::config::OptionalColor;
use crate::CONFIG;
use std::convert::From;
use todo_txt::Task;
use tui::style::Style;
use tui::text::Span;
use tui::widgets::ListItem;
use std::ops::Index;

pub struct TaskList<'a>(pub Vec<(usize, &'a Task)>);

impl<'a> TaskList<'a> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> Index<usize> for TaskList<'a> {
    type Output = Task;
    fn index<'b>(&'b self, i: usize) -> &'a Task {
        &self.0[i].1
    }
}

impl<'a> Into<Vec<ListItem<'a>>> for TaskList<'a> {
    fn into(self) -> Vec<ListItem<'a>> {
        self.0
            .iter()
            .map(|task| {
                match CONFIG.priority_colors[usize::from(u8::from(task.1.priority.clone()))] {
                    OptionalColor::Some(color) => ListItem::new(Span::styled(
                        task.1.subject.clone(),
                        Style::default().fg(color),
                    )),
                    OptionalColor::Default => ListItem::new(task.1.subject.clone()),
                }
            })
            .collect::<Vec<ListItem<'a>>>()
    }
}
