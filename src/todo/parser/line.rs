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

    pub fn fill(&self, todo: &ToDo) -> Vec<(String, Style)> {
        self.0.iter().map_while(|block| block.fill(todo)).collect()
    }
}

