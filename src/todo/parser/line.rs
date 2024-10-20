use super::LineBlock;

use super::ToDo;
use crate::{config::Styles, Result};
use tui::style::Style;

#[derive(Default, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Line(pub Vec<LineBlock>);

impl Line {
    pub fn add_span_styled(
        &mut self,
        parts: &str,
        style: Option<String>,
        styles: &Styles,
    ) -> Result<()> {
        if !parts.is_empty() {
            self.0
                .push(LineBlock::try_from_styled(parts, style, styles)?);
        }
        Ok(())
    }

    pub fn fill(&self, todo: &ToDo, styles: &Styles) -> Option<Vec<(String, Style)>> {
        if self.0.is_empty() {
            return None;
        }
        let ret: Vec<(String, Style)> = self
            .0
            .iter()
            .filter_map(|block| block.fill(todo, styles))
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

        assert_eq!(
            Line(vec![
                LineBlock::try_from_styled("some text", None, &styles).unwrap(),
                LineBlock::try_from_styled("not empty $done", None, &styles).unwrap(),
            ])
            .fill(&todo, &styles),
            Some(vec![
                (String::from("some text"), Style::default()),
                (String::from("not empty 0"), Style::default())
            ])
        );

        assert_eq!(
            Line(vec![
                LineBlock::try_from_styled("some text", None, &styles).unwrap(),
                LineBlock::try_from_styled("empty $priority", None, &styles).unwrap(),
            ])
            .fill(&todo, &styles),
            Some(vec![(String::from("some text"), Style::default())])
        );
    }
}
