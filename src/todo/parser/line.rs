use super::{LineBlock, ToDo};
use crate::{config::Styles, Result};
use todo_txt::Task;
use tui::style::Style;

#[derive(Default, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Line(pub Vec<LineBlock>);

impl Line {
    pub fn add_span_styled(
        &mut self,
        parts: &str,
        style: Option<String>,
        to_colorize: bool,
        styles: &Styles,
    ) -> Result<()> {
        if !parts.is_empty() {
            self.0.push(LineBlock::try_from_styled(
                parts,
                style,
                to_colorize,
                styles,
            )?);
        }
        Ok(())
    }

    pub fn fill(&self, task: &Task, todo: &ToDo, styles: &Styles) -> Option<Vec<(String, Style)>> {
        let ret: Vec<_> = self
            .0
            .iter()
            .filter_map(|block| block.fill(task, todo, styles))
            .flatten()
            .filter(|(part, _style)| !part.is_empty())
            .collect();
        if ret.is_empty() {
            None
        } else {
            Some(ret)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::todo::ToDoData;

    use super::*;

    #[test]
    fn line_fill() {
        let styles = Styles::default();
        let mut todo = ToDo::default();
        todo.new_task("Some task 1").unwrap();
        todo.new_task("Some task 2").unwrap();
        todo.new_task("Some task 3").unwrap();
        todo.set_active(ToDoData::Pending, 0);

        let task = Task::from_str("Some task 1").unwrap();

        assert_eq!(
            Line(vec![
                LineBlock::try_from_styled("some text", None, false, &styles).unwrap(),
                LineBlock::try_from_styled("not empty $done", None, false, &styles).unwrap(),
            ])
            .fill(&task, &todo, &styles),
            Some(vec![
                (String::from("some text"), Style::default()),
                (String::from("not empty 0"), Style::default())
            ])
        );

        assert_eq!(
            Line(vec![
                LineBlock::try_from_styled("some text", None, false, &styles).unwrap(),
                LineBlock::try_from_styled("empty $priority", None, false, &styles).unwrap(),
            ])
            .fill(&task, &todo, &styles),
            Some(vec![(String::from("some text"), Style::default())])
        );
    }
}
