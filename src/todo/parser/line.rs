use super::LineBlock;

use super::ToDo;
use crate::error::ToDoRes;
use tui::style::Style;

#[derive(Default, PartialEq, Eq, Debug)]
pub struct Line(pub Vec<LineBlock>);

impl Line {
    pub fn add_span(&mut self, parts: &str) -> ToDoRes<()> {
        self.add_span_styled(parts, None)
    }

    pub fn add_span_styled(&mut self, parts: &str, style: Option<String>) -> ToDoRes<()> {
        if !parts.is_empty() {
            self.0.push(LineBlock::try_from_styled(parts, style)?);
        }
        Ok(())
    }

    pub fn fill(&self, todo: &ToDo) -> Option<Vec<(String, Style)>> {
        if self.0.is_empty() {
            return None
        } 
        let ret: Vec<(String, Style)> = self.0.iter().filter_map(|block| block.fill(todo)).collect();
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
        let mut todo = ToDo::default();
        todo.new_task("Some task 1").unwrap();
        todo.new_task("Some task 2").unwrap();
        todo.new_task("Some task 3").unwrap();
        todo.set_active(ToDoData::Pending, 0);

        assert_eq!(
            Line(vec![
                LineBlock::try_from_styled("some text", None).unwrap(),
                LineBlock::try_from_styled("not empty $done", None).unwrap(),
            ])
            .fill(&todo),
            Some(vec![
                (String::from("some text"), Style::default()),
                (String::from("not empty 0"), Style::default())
            ])
        );


        assert_eq!(
            Line(vec![
                LineBlock::try_from_styled("some text", None).unwrap(),
                LineBlock::try_from_styled("empty $priority", None).unwrap(),
            ])
            .fill(&todo),
            Some(vec![
                (String::from("some text"), Style::default())
            ])
        );
    }
}
