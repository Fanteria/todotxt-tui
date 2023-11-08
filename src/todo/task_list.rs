use crate::CONFIG;
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::ops::Index;
use todo_txt::Task;
use tui::text::Line;
use tui::text::Span;
use tui::widgets::ListItem;

type Item<'a> = (usize, &'a Task);

/// Represents the possible sorting options for tasks.
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

    /// Retrieves the actual index of a task based on its position in the list.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the task in the list.
    ///
    /// # Returns
    ///
    /// The actual index of the task in the original list.
    pub fn get_actual_index(&self, index: usize) -> usize {
        self.0[index].0
    }

    /// Slices the task list from `first` (inclusive) to `last` (exclusive).
    ///
    /// # Arguments
    ///
    /// * `first` - The index of the first task to include in the slice.
    /// * `last` - The index of the first task to exclude from the slice.
    ///
    /// # Returns
    ///
    /// A `TaskSlice` containing the sliced tasks.
    pub fn slice(&self, first: usize, last: usize) -> TaskSlice {
        if last > self.0.len() {
            return TaskSlice(&self.0[first..]);
        };
        TaskSlice(&self.0[first..last])
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

    /// Parses a task's string representation into a vector of `Span` elements for rendering.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to parse.
    ///
    /// # Returns
    ///
    /// A vector of `Span` elements representing the parsed task.
    pub fn parse_task_string(task: &Task) -> Vec<Span> {
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

        let style = CONFIG
            .priority_colors
            .get_style(usize::from(u8::from(task.priority.clone())));

        if indexes.is_empty() {
            return vec![Span::styled(&task.subject, style)];
        }

        indexes.sort_by(|a, b| a.0.cmp(&b.0));

        let get_style = |category: &str| {
            let style_category = match category.chars().next().unwrap() {
                '+' => CONFIG.projects_style.combine(&CONFIG.category_style),
                '@' => CONFIG.contexts_style.combine(&CONFIG.category_style),
                '#' => CONFIG.hashtags_style.combine(&CONFIG.category_style),
                _ => CONFIG.category_style,
            };
            match CONFIG.custom_category_style.get(category) {
                Some(style_custom) => style_category.combine(style_custom).get_style(),
                None => style_category.get_style(),
            }
        };

        let mut parsed = vec![Span::styled(&task.subject[0..indexes[0].0], style)];
        indexes.iter().zip(indexes.iter().skip(1)).for_each(
            |((act_index, act_len), (next_index, _))| {
                let end_index = act_index + act_len;
                let s = &task.subject[*act_index..end_index];
                parsed.push(Span::styled(s, get_style(s)));
                parsed.push(Span::styled(&task.subject[end_index..*next_index], style));
            },
        );
        let (last_index, last_len) = indexes.last().unwrap();
        let s = &task.subject[*last_index..last_index + last_len];
        parsed.push(Span::styled(s, get_style(s)));

        parsed
    }
}

impl<'a> Index<usize> for TaskList<'a> {
    type Output = Task;
    fn index<'b>(&'b self, i: usize) -> &'a Task {
        self.0[i].1
    }
}

impl<'a> From<TaskSlice<'a>> for Vec<ListItem<'a>> {
    fn from(val: TaskSlice<'a>) -> Self {
        val.0
            .iter()
            .map(|(_, task)| ListItem::new(Line::from(TaskList::parse_task_string(task))))
            .collect::<Vec<ListItem<'a>>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_task_string() {
        let task = Task::from_str("measure space for 1 +project1 ~ @context1 #hashtag1").unwrap();
        let parsed = TaskList::parse_task_string(&task);
        assert_eq!(parsed[0].content, "measure space for 1 ");
        assert_eq!(parsed[1].content, "+project1");
        assert_eq!(parsed[2].content, " ~ ");
        assert_eq!(parsed[3].content, "@context1");
        assert_eq!(parsed[4].content, " ");
        assert_eq!(parsed[5].content, "#hashtag1");
    }

    #[test]
    fn task_slice() {
        let task1 = Task::from_str("measure space for 1").unwrap();
        let task2 = Task::from_str("measure space for 2").unwrap();
        let task3 = Task::from_str("measure space for 3").unwrap();
        let task4 = Task::from_str("measure space for 4").unwrap();
        let tasklist = TaskList(vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)]);
        let slice = tasklist.slice(1, 3);

        assert_eq!(slice.0.len(), 2);
        assert_eq!(slice.0[0], (1, &task2));
        assert_eq!(slice.0[1], (2, &task3));

        let slice = tasklist.slice(1, 100_000);
        assert_eq!(slice.0.len(), 3);
        assert_eq!(slice.0[0], (1, &task2));
        assert_eq!(slice.0[1], (2, &task3));
        assert_eq!(slice.0[2], (3, &task4));
    }

    #[test]
    fn sort_tasklist() {
        let compare = |expected: &TaskList, real: TaskList| {
            assert_eq!(expected.0.len(), real.0.len());
            for i in 0..expected.0.len() {
                assert_eq!(expected[i], real[i]);
            }
        };
        let task1 = Task::from_str("(C) 2 measure space for 1").unwrap();
        let task2 = Task::from_str("    3 measure space for 2").unwrap();
        let task3 = Task::from_str("    1 measure space for 3").unwrap();
        let task4 = Task::from_str("(A) 4 measure space for 4").unwrap();
        let tasklist = TaskList(vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)]);

        let mut none = TaskList(vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)]);
        none.sort(TaskSort::None);
        compare(&tasklist, none);

        let mut reverse = TaskList(vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)]);
        reverse.sort(TaskSort::Reverse);
        compare(&TaskList(vec![(3, &task4), (2, &task3), (1, &task2), (0, &task1)]), reverse);

        let mut priority = TaskList(vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)]);
        priority.sort(TaskSort::Priority);
        compare(&TaskList(vec![(3, &task4), (0, &task1), (1, &task2), (2, &task3)]), priority);

        let mut alpha = TaskList(vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)]);
        alpha.sort(TaskSort::Alphanumeric);
        compare(&TaskList(vec![(2, &task3), (0, &task1), (1, &task2), (3, &task4)]), alpha);

        let mut alpha_reverse = TaskList(vec![(0, &task1), (1, &task2), (2, &task3), (3, &task4)]);
        alpha_reverse.sort(TaskSort::AlphanumericReverse);
        compare(&TaskList(vec![(3, &task4), (1, &task2), (0, &task1), (2, &task3)]), alpha_reverse);
    }

}
