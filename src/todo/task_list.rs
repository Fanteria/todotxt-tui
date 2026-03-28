use crate::config::{Styles, TaskSort};
use std::{
    convert::From,
    ops::{Bound, Index, RangeBounds},
};
use todo_txt::task::Simple as Task;
use tui::{
    text::{Line, Span},
    widgets::{List, ListItem},
};

use super::{
    search::{Search, Searchable},
    Parser, ToDo,
};

type Item<'a> = (usize, &'a Task);

/// Represents a list of tasks, where each task is a tuple of `(usize, &'a Task)`.
/// The `usize` value is the index of the task in the original list.
pub struct TaskList<'a, 'b> {
    vec: Vec<Item<'a>>,
    styles: &'b Styles,
}

pub struct TaskView<'a> {
    vec: &'a [Item<'a>],
    styles: &'a Styles,
    to_search: Option<&'a str>,
    parser: &'a Parser,
    todo: &'a ToDo,
}

impl<'a, 'b> TaskList<'a, 'b> {
    /// Creates a new `TaskList` instance.
    pub fn new(vec: Vec<Item<'a>>, styles: &'b Styles) -> Self {
        Self { vec, styles }
    }

    /// Returns the number of tasks in the list.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Checks if the task list is empty.
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Retrieves the actual index of a task based on its position in the list.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the task in the list.
    ///
    /// # Returns
    ///
    /// The actual index of the task in the original list.
    pub fn get_actual_index(&self, index: usize) -> Option<usize> {
        Some(self.vec.get(index)?.0)
    }

    /// Slices the task list based on the provided range of indexes and returns
    /// a view of the tasks.
    ///
    /// # Arguments
    ///
    /// * `range` - A range of indexes specifying the start and end points of the slice.
    /// * `to_search` - An optional search string used to highlight tasks.
    ///
    /// # Returns
    ///
    /// A `TaskView` containing the sliced tasks and relevant styling, limited
    /// to the specified range.
    pub fn get_view(
        &'a self,
        range: impl RangeBounds<usize>,
        to_search: Option<&'a str>,
        parser: &'a Parser,
        todo: &'a ToDo,
    ) -> TaskView<'a> {
        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&n) => std::cmp::min(n + 1, self.vec.len()),
            Bound::Excluded(&n) => std::cmp::min(n, self.vec.len()),
            Bound::Unbounded => self.vec.len(),
        };
        TaskView {
            vec: &self.vec[start..end],
            styles: self.styles,
            to_search,
            parser,
            todo,
        }
    }

    /// Sorts the task list based on the specified sorting criteria.
    ///
    /// # Arguments
    ///
    /// * `sort` - The sorting criteria to apply.
    pub fn sort(&mut self, sort: &[TaskSort]) {
        sort.iter().rev().for_each(|sort| match sort {
            TaskSort::Reverse => self.vec.reverse(),
            TaskSort::Priority => self
                .vec
                .sort_by(|(_, a), (_, b)| b.priority.cmp(&a.priority)),
            TaskSort::Alphanumeric => self.vec.sort_by(|(_, a), (_, b)| a.subject.cmp(&b.subject)),
            TaskSort::AlphanumericReverse => {
                self.vec.sort_by(|(_, a), (_, b)| b.subject.cmp(&a.subject))
            }
            TaskSort::CreateDateAsc => self
                .vec
                .sort_by_key(|(_, t)| (t.create_date.is_none(), t.create_date)),
            TaskSort::CreateDateDesc => self.vec.sort_by_key(|(_, t)| {
                (
                    t.create_date.is_none(),
                    t.create_date.map(std::cmp::Reverse),
                )
            }),
            TaskSort::FinishDataAsc => self
                .vec
                .sort_by_key(|(_, t)| (t.finish_date.is_none(), t.finish_date)),
            TaskSort::FinishDataDesc => self.vec.sort_by_key(|(_, t)| {
                (
                    t.finish_date.is_none(),
                    t.finish_date.map(std::cmp::Reverse),
                )
            }),
            TaskSort::DueDateAsc => self
                .vec
                .sort_by_key(|(_, t)| (t.due_date.is_none(), t.due_date)),
            TaskSort::DueDateDesc => self
                .vec
                .sort_by_key(|(_, t)| (t.due_date.is_none(), t.due_date.map(std::cmp::Reverse))),
        });
    }
}

impl<'a, 'b> Index<usize> for TaskList<'a, 'b> {
    type Output = Task;
    fn index<'c>(&'c self, i: usize) -> &'a Task {
        self.vec[i].1
    }
}

impl<'a, 'b> Searchable for TaskList<'a, 'b> {
    fn search_through(&self) -> impl DoubleEndedIterator + ExactSizeIterator<Item = &str> {
        self.vec.iter().map(|item| item.1.subject.as_str())
    }
}

impl<'a> From<TaskView<'a>> for List<'_> {
    fn from(val: TaskView<'a>) -> Self {
        List::new(val.vec.iter().map(|(_, task)| {
            let parsed = val.parser.fill(task, val.todo);

            let lines: Vec<Line> = parsed
                .iter()
                .map(|line| {
                    Line::from(
                        line.iter()
                            .flat_map(|(text, style)| {
                                Search::highlight(text, val.to_search, val.styles, *style)
                            })
                            // Span must be owned
                            .map(|span| Span::styled(span.content.to_string(), span.style))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect();

            ListItem::new(lines)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use std::str::FromStr;

    #[test]
    fn task_slice() -> Result<()> {
        let styles = Styles::default();
        let task1 = Task::from_str("measure space for 1")?;
        let task2 = Task::from_str("measure space for 2")?;
        let task3 = Task::from_str("measure space for 3")?;
        let task4 = Task::from_str("measure space for 4")?;
        let tasklist = TaskList {
            vec: vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)],
            styles: &styles,
        };
        let todo_list = ToDo::default();
        let parser = Parser::new("", Styles::default()).unwrap();
        let slice = tasklist.get_view(1..3, None, &parser, &todo_list);

        assert_eq!(slice.vec.len(), 2);
        assert_eq!(slice.vec[0], (1, &task2));
        assert_eq!(slice.vec[1], (2, &task3));

        let slice = tasklist.get_view(1..100_000, None, &parser, &todo_list);
        assert_eq!(slice.vec.len(), 3);
        assert_eq!(slice.vec[0], (1, &task2));
        assert_eq!(slice.vec[1], (2, &task3));
        assert_eq!(slice.vec[2], (3, &task4));

        Ok(())
    }

    #[test]
    fn sort_tasklist() -> Result<()> {
        let task0 = Task::from_str("(C) 2026-03-25 2 measure space for 1")?;
        let task1 = Task::from_str("2026-03-24 3 measure space for 2")?;
        let task2 = Task::from_str("1 measure space for 3")?;
        let task3 = Task::from_str("(A) 2026-03-22 4 measure space for 4")?;

        let get_sorted = |filters| {
            let mut list = TaskList {
                vec: vec![(0, &task0), (1, &task1), (2, &task2), (3, &task3)],
                styles: &Styles::default(),
            };
            list.sort(filters);
            list.vec
        };

        assert_eq!(
            vec![(0, &task0), (1, &task1), (2, &task2), (3, &task3)],
            get_sorted(&[])
        );

        assert_eq!(
            vec![(3, &task3), (2, &task2), (1, &task1), (0, &task0)],
            get_sorted(&[TaskSort::Reverse])
        );

        assert_eq!(
            vec![(3, &task3), (0, &task0), (1, &task1), (2, &task2)],
            get_sorted(&[TaskSort::Priority])
        );

        assert_eq!(
            vec![(2, &task2), (0, &task0), (1, &task1), (3, &task3)],
            get_sorted(&[TaskSort::Alphanumeric])
        );

        assert_eq!(
            vec![(3, &task3), (1, &task1), (0, &task0), (2, &task2)],
            get_sorted(&[TaskSort::AlphanumericReverse])
        );

        assert_eq!(
            vec![(3, &task3), (0, &task0), (2, &task2), (1, &task1)],
            get_sorted(&[TaskSort::Priority, TaskSort::Alphanumeric])
        );

        assert_eq!(
            vec![(3, &task3), (1, &task1), (0, &task0), (2, &task2)],
            get_sorted(&[TaskSort::CreateDateAsc])
        );

        assert_eq!(
            vec![(0, &task0), (1, &task1), (3, &task3), (2, &task2)],
            get_sorted(&[TaskSort::CreateDateDesc])
        );

        Ok(())
    }
}
