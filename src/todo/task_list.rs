use crate::config::{Styles, TaskSort};
use std::convert::From;
use std::ops::Bound;
use std::ops::Index;
use std::ops::RangeBounds;
use todo_txt::Task;
use tui::text::Line;
use tui::text::Span;
use tui::widgets::ListItem;

use super::search::Search;

type Item<'a> = (usize, &'a Task);

/// Represents a list of tasks, where each task is a tuple of `(usize, &'a Task)`.
/// The `usize` value is the index of the task in the original list.
pub struct TaskList<'a> {
    pub vec: Vec<Item<'a>>,
    pub styles: &'a Styles,
}

pub struct TaskView<'a> {
    pub vec: &'a [Item<'a>],
    pub styles: &'a Styles,
    pub to_search: Option<&'a str>,
}

impl<'a> TaskList<'a> {
    /// Returns the number of tasks in the list.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

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
    ) -> TaskView {
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
        }
    }

    /// Sorts the task list based on the specified sorting criteria.
    ///
    /// # Arguments
    ///
    /// * `sort` - The sorting criteria to apply.
    pub fn sort(&mut self, sort: TaskSort) {
        use TaskSort::*;
        match sort {
            None => {}
            Reverse => self.vec.reverse(),
            Priority => self
                .vec
                .sort_by(|(_, a_task), (_, b_task)| b_task.priority.cmp(&a_task.priority)),
            Alphanumeric => self
                .vec
                .sort_by(|(_, a_task), (_, b_task)| a_task.subject.cmp(&b_task.subject)),
            AlphanumericReverse => self
                .vec
                .sort_by(|(_, a_task), (_, b_task)| b_task.subject.cmp(&a_task.subject)),
        }
    }

    /// Parses a task's string representation into a vector of `Span` elements for rendering.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to parse.
    ///
    /// # Returns
    ///
    /// A vector of `Span` elements representing the parsed task.
    pub fn parse_task_string(
        task: &'a Task,
        styles: &'a Styles,
        to_search: Option<&str>,
    ) -> Vec<Span<'a>> {
        let style = styles
            .priority_style
            .get_style(u8::from(task.priority.clone()));
        let mut indexes = Vec::new();

        let mut collect_indexes = |separator, iter: core::slice::Iter<'_, String>| {
            iter.for_each(|project| {
                indexes.push((
                    task.subject
                        .find(&(String::from(separator) + project))
                        .unwrap(),
                    project.len() + 1,
                ));
            });
        };

        collect_indexes('+', task.projects().iter());
        collect_indexes('@', task.contexts().iter());
        collect_indexes('#', task.hashtags.iter());

        if indexes.is_empty() {
            return Search::highlight(&task.subject, to_search, styles, style);
        }

        indexes.sort_by(|a, b| a.0.cmp(&b.0));

        let mut parsed =
            Search::highlight(&task.subject[0..indexes[0].0], to_search, styles, style);
        indexes.iter().zip(indexes.iter().skip(1)).for_each(
            |((act_index, act_len), (next_index, _))| {
                let end_index = act_index + act_len;
                let s = &task.subject[*act_index..end_index];
                parsed.push(Span::styled(s, styles.get_category_style(s).get_style()));
                parsed.push(Span::styled(&task.subject[end_index..*next_index], style));
            },
        );
        let (last_index, last_len) = indexes.last().unwrap();
        let s = &task.subject[*last_index..last_index + last_len];
        parsed.push(Span::styled(s, styles.get_category_style(s).get_style()));

        parsed
    }
}

impl<'a> Index<usize> for TaskList<'a> {
    type Output = Task;
    fn index<'b>(&'b self, i: usize) -> &'a Task {
        self.vec[i].1
    }
}

impl<'a> From<TaskView<'a>> for Vec<ListItem<'a>> {
    fn from(val: TaskView<'a>) -> Self {
        val.vec
            .iter()
            .map(|(_, task)| {
                ListItem::new(Line::from(TaskList::parse_task_string(
                    task,
                    val.styles,
                    val.to_search,
                )))
            })
            .collect::<Vec<ListItem<'a>>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_task_string() {
        let styles = Styles::default();
        let task = Task::from_str("measure space for 1 +project1 ~ @context1 #hashtag1").unwrap();
        let parsed = TaskList::parse_task_string(&task, &styles, None);
        assert_eq!(parsed[0].content, "measure space for 1 ");
        assert_eq!(parsed[1].content, "+project1");
        assert_eq!(parsed[2].content, " ~ ");
        assert_eq!(parsed[3].content, "@context1");
        assert_eq!(parsed[4].content, " ");
        assert_eq!(parsed[5].content, "#hashtag1");
    }

    #[test]
    fn task_slice() {
        let styles = Styles::default();
        let task1 = Task::from_str("measure space for 1").unwrap();
        let task2 = Task::from_str("measure space for 2").unwrap();
        let task3 = Task::from_str("measure space for 3").unwrap();
        let task4 = Task::from_str("measure space for 4").unwrap();
        let tasklist = TaskList {
            vec: vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)],
            styles: &styles,
        };
        let slice = tasklist.get_view(1..3, None);

        assert_eq!(slice.vec.len(), 2);
        assert_eq!(slice.vec[0], (1, &task2));
        assert_eq!(slice.vec[1], (2, &task3));

        let slice = tasklist.get_view(1..100_000, None);
        assert_eq!(slice.vec.len(), 3);
        assert_eq!(slice.vec[0], (1, &task2));
        assert_eq!(slice.vec[1], (2, &task3));
        assert_eq!(slice.vec[2], (3, &task4));
    }

    #[test]
    fn sort_tasklist() {
        let compare = |expected: &TaskList, real: TaskList| {
            assert_eq!(expected.len(), real.len());
            for i in 0..expected.len() {
                assert_eq!(expected[i], real[i]);
            }
        };
        let styles = Styles::default();
        let task1 = Task::from_str("(C) 2 measure space for 1").unwrap();
        let task2 = Task::from_str("    3 measure space for 2").unwrap();
        let task3 = Task::from_str("    1 measure space for 3").unwrap();
        let task4 = Task::from_str("(A) 4 measure space for 4").unwrap();
        let tasklist = TaskList {
            vec: vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)],
            styles: &styles,
        };

        let mut none = TaskList {
            vec: vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)],
            styles: &styles,
        };
        none.sort(TaskSort::None);
        compare(&tasklist, none);

        let mut reverse = TaskList {
            vec: vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)],
            styles: &styles,
        };
        reverse.sort(TaskSort::Reverse);
        compare(
            &TaskList {
                vec: vec![(3, &task4), (2, &task3), (1, &task2), (0, &task1)],
                styles: &styles,
            },
            reverse,
        );

        let mut priority = TaskList {
            vec: vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)],
            styles: &styles,
        };
        priority.sort(TaskSort::Priority);
        compare(
            &TaskList {
                vec: vec![(3, &task4), (0, &task1), (1, &task2), (2, &task3)],
                styles: &styles,
            },
            priority,
        );

        let mut alpha = TaskList {
            vec: vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)],
            styles: &styles,
        };
        alpha.sort(TaskSort::Alphanumeric);
        compare(
            &TaskList {
                vec: vec![(2, &task3), (0, &task1), (1, &task2), (3, &task4)],
                styles: &styles,
            },
            alpha,
        );

        let mut alpha_reverse = TaskList {
            vec: vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)],
            styles: &styles,
        };
        alpha_reverse.sort(TaskSort::AlphanumericReverse);
        compare(
            &TaskList {
                vec: vec![(3, &task4), (1, &task2), (0, &task1), (2, &task3)],
                styles: &styles,
            },
            alpha_reverse,
        );
    }
}
