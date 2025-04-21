use super::{
    parts::{PartStyle, Parts},
    ToDo,
};
use crate::config::Styles;
use regex::Regex;
use std::sync::LazyLock;
use todo_txt::Task;
use tui::style::Style;

static REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s)([+@#][^\s]+)").unwrap());

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct LineBlock {
    pub parts: Vec<Parts>,
    pub style: PartStyle,
}

impl LineBlock {
    pub fn new(parts: Vec<Parts>, part_style: Option<&PartStyle>) -> Self {
        Self {
            parts,
            style: part_style.cloned().unwrap_or_default(),
        }
    }

    pub fn fill(&self, task: &Task, todo: &ToDo, styles: &Styles) -> Option<Vec<(String, Style)>> {
        let style = self.style.get_style(task, styles).get_style();
        let string = self
            .parts
            .iter()
            .map(|part| part.fill(task, todo))
            .collect::<Option<String>>()?;
        if !self.style.to_colorize {
            return Some(vec![(string, style)]);
        }
        let mut last_index = 0;
        Some(
            REGEX
                .captures_iter(&string)
                .filter_map(|m| m.get(1))
                .flat_map(|m| {
                    let start = last_index;
                    last_index = m.end();
                    let category = string[m.start()..m.end()].to_string(); let category_style = styles.get_category_style(&category).get_style();
                    [
                        (string[start..m.start()].to_string(), style),
                        (category, category_style),
                    ]
                })
                .collect::<Vec<_>>()
                .into_iter()
                .chain([(string[last_index..string.len()].to_string(), style)])
                .collect::<Vec<_>>(),
        )
    }
}
