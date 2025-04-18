mod line;
mod line_block;
mod parts;

use super::{ToDo, ToDoData};
use crate::{config::Styles, Result};
use line::Lines;
use line_block::LineBlock;
use std::str::FromStr;
use todo_txt::Task;
use tui::style::Style;

#[derive(pest_derive::Parser)]
#[grammar = "./todo/parser/grammar.pest"]
struct ParserGrammar;

#[derive(Debug)]
pub struct Parser {
    lines: Lines,
    styles: Styles,
}

impl Parser {
    pub fn new(value: &str, styles: Styles) -> Result<Self> {
        let lines = Lines::from_str(value)?;
        log::debug!("Loaded parser: {:#?}", lines);
        Ok(Parser { lines, styles })
    }

    pub fn fill(&self, task: &Task, todo: &ToDo) -> Vec<Vec<(String, Style)>> {
        self.lines
            .iter()
            .filter_map(|line| line.fill(task, todo, &self.styles))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::config::Conf;

    #[test]
    fn fill() -> Result<()> {
        let mut todo = ToDo::default();
        todo.new_task("Project +project asdf").unwrap();
        todo.new_task("Some task 2").unwrap();
        todo.new_task("Some task 3").unwrap();
        let task = Task::from_str("task")?;
        let style = Styles::from_reader(
            r#"
[projects_style]
fg = "Green"

[contexts_style]
fg = "Red"

[hashtags_style]
fg = "Yellow"

[custom_category_style."+custom"]
fg = "Blue"
        "#
            .as_bytes(),
        )
        .unwrap();

        let lines = Lines::from_str(
            "[Done +project to be @splitted here #hashtag and some +custom project](! ^black)",
        )?;

        use tui::style::Color;
        assert_eq!(
            lines[0][0].fill(&task, &todo, &style),
            Some(vec![
                (String::from("Done "), Style::default().bg(Color::Black)),
                (String::from("+project"), Style::default().fg(Color::Green)),
                (String::from(" to be "), Style::default().bg(Color::Black)),
                (String::from("@splitted"), Style::default().fg(Color::Red)),
                (String::from(" here "), Style::default().bg(Color::Black)),
                (String::from("#hashtag"), Style::default().fg(Color::Yellow)),
                (
                    String::from(" and some "),
                    Style::default().bg(Color::Black)
                ),
                (String::from("+custom"), Style::default().fg(Color::Blue)),
                (String::from(" project"), Style::default().bg(Color::Black)),
            ])
        );

        Ok(())
    }

    #[test]
    fn fill_base() -> Result<()> {
        let parser = Parser::new("some text", Styles::default())?;
        let mut todo = ToDo::default();
        todo.new_task("task").unwrap();
        todo.new_task("x done task").unwrap();

        let task = Task::from_str("task").unwrap();

        assert_eq!(
            parser.fill(&task, &todo),
            vec![vec![(String::from("some text"), Style::default())]]
        );

        todo.set_active(ToDoData::Pending, 0);
        assert_eq!(
            parser.fill(&task, &todo),
            vec![vec![(String::from("some text"), Style::default())]]
        );

        Ok(())
    }

    #[test]
    fn fill_counts() -> Result<()> {
        let parser = Parser::new("Done: $done Pending: $pending", Styles::default())?;
        let mut todo = ToDo::default();
        todo.new_task("task").unwrap();
        todo.new_task("x done task").unwrap();
        todo.set_active(ToDoData::Pending, 0);
        let task = Task::from_str("task").unwrap();

        assert_eq!(
            parser.fill(&task, &todo),
            vec![vec![(String::from("Done: 1 Pending: 1"), Style::default())]]
        );

        Ok(())
    }
}
