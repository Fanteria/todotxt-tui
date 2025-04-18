use super::{
    parts::{PartStyle, Parts},
    LineBlock, ToDo,
};
use crate::{
    config::Styles,
    todo::parser::{ParserGrammar, Rule},
    ToDoError,
};
use pest::{iterators::Pairs, Parser};
use std::{ops::Deref, str::FromStr};
use todo_txt::Task;
use tui::style::Style;

#[derive(Default, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Line(Vec<LineBlock>);

impl Line {
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

impl Deref for Line {
    type Target = Vec<LineBlock>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Lines(Vec<Line>);

impl Deref for Lines {
    type Target = Vec<Line>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Lines {
    type Err = ToDoError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        fn parse_parts(parts: Pairs<'_, Rule>, style: Option<PartStyle>) -> Vec<LineBlock> {
            let mut blocks = vec![];
            let mut block_parts = Vec::new();
            parts.for_each(|part| match part.as_rule() {
                Rule::text => block_parts.push(Parts::Text(part.as_str().to_string())),
                Rule::var_pending => block_parts.push(Parts::Pending),
                Rule::var_done => block_parts.push(Parts::Done),
                Rule::var_subject => block_parts.push(Parts::Subject),
                Rule::var_priority => block_parts.push(Parts::Priority),
                Rule::var_create_date => block_parts.push(Parts::CreateDate),
                Rule::var_finish_date => block_parts.push(Parts::FinishDate),
                Rule::var_finished => block_parts.push(Parts::Finished),
                Rule::var_treshold_date => block_parts.push(Parts::TresholdDate),
                Rule::var_due_date => block_parts.push(Parts::DueDate),
                Rule::var_contexts => block_parts.push(Parts::Contexts),
                Rule::var_projects => block_parts.push(Parts::Projects),
                Rule::var_hashtags => block_parts.push(Parts::Hashtags),
                Rule::var_link => block_parts.push(Parts::Special("link".into())),
                Rule::block => {
                    let mut inner = part.into_inner();
                    if !block_parts.is_empty() {
                        blocks.push(LineBlock::new(
                            std::mem::take(&mut block_parts),
                            style.as_ref(),
                        ));
                    }
                    blocks.append(&mut parse_parts(
                        inner.next().unwrap().into_inner(),
                        inner
                            .next()
                            .map(|pairs| PartStyle::try_from(pairs.into_inner()).unwrap()),
                    ));
                }
                _ => unreachable!(),
            });
            blocks.push(LineBlock::new(block_parts, style.as_ref()));
            blocks
        }
        let lines = ParserGrammar::parse(Rule::lines, s)
            .map_err(|e| ToDoError::FailedToParseParser(Box::new(e)))?
            .next()
            .unwrap() // Safe. Already parsed by pest.
            .into_inner()
            .filter_map(|line| {
                if line.as_rule() == Rule::line {
                    Some(line.into_inner())
                } else {
                    None
                }
            })
            .map(|parts| Line(parse_parts(parts, None)))
            .collect();
        Ok(Lines(lines))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Color, TextStyle};
    use crate::error::Result;
    use crate::todo::ToDoData;
    use std::str::FromStr;

    #[test]
    fn line_fill() -> Result<()> {
        let styles = Styles::default();
        let mut todo = ToDo::default();
        todo.new_task("Some task 1").unwrap();
        todo.new_task("Some task 2").unwrap();
        todo.new_task("Some task 3").unwrap();
        todo.set_active(ToDoData::Pending, 0);

        let task = Task::from_str("Some task 1").unwrap();

        assert_eq!(
            Lines::from_str("some text[not empty $done]")?[0].fill(&task, &todo, &styles),
            Some(vec![
                (String::from("some text"), Style::default()),
                (String::from("not empty 0"), Style::default())
            ])
        );

        assert_eq!(
            Lines::from_str("some text[empty $priority]")?[0].fill(&task, &todo, &styles),
            Some(vec![(String::from("some text"), Style::default())])
        );

        Ok(())
    }

    #[test]
    fn parse_variables() -> Result<()> {
        assert_eq!(Lines::from_str("")?[0][0].parts, vec![]);

        assert_eq!(
            Lines::from_str("some text")?[0][0].parts,
            vec![Parts::Text("some text".into())]
        );

        assert_eq!(
            Lines::from_str("some text $done another text")?[0][0].parts,
            vec![
                Parts::Text("some text ".into()),
                Parts::Done,
                Parts::Text(" another text".into())
            ]
        );

        assert_eq!(
            Lines::from_str("there is ${pending}x pending tasks")?[0][0].parts,
            vec![
                Parts::Text("there is ".into()),
                Parts::Pending,
                Parts::Text("x pending tasks".into())
            ]
        );

        // TODO this test failing
        // assert_eq!(
        //     Lines::from_str("special task text $some-special")?[0][0].parts,
        //     vec![Parts::Text("special task text ".into()), Parts::Special("some-special".into())]
        // );

        // TODO this test failing
        // assert_eq!(
        //     Lines::from_str("special \\$ character")?[0][0].parts,
        //     vec![Parts::Text("special $ character".into())]
        // );

        assert_eq!(
            Lines::from_str("Pending: $pending Done: $done")?[0][0].parts,
            vec![
                Parts::Text("Pending: ".into()),
                Parts::Pending,
                Parts::Text(" Done: ".into()),
                Parts::Done
            ]
        );

        Ok(())
    }

    #[test]
    fn parse_variables_error() {
        assert!(Lines::from_str("string with $ empty variable").is_err());
        assert!(Lines::from_str("string with empty variable on end $").is_err());
        assert!(Lines::from_str("variable block not closed ${variable ").is_err());
    }

    #[test]
    fn parse() -> Result<()> {
        let task = Task::default();

        assert!(Lines::from_str("")?[0][0].parts.is_empty());
        assert_eq!(
            Lines::from_str("some text")?[0][0].parts,
            vec![Parts::Text("some text".to_string())]
        );

        // TODO failed
        // assert_eq!(
        //     Lines::from_str("some text \\[ with escapes \\]")?[0][0].parts,
        //     vec![Parts::Text("some text [ with escapes ]".to_string())]
        // );

        let lines = Lines::from_str("[some text](Red)")?;
        assert_eq!(
            lines[0][0].parts,
            vec![Parts::Text("some text".to_string())]
        );
        // Line(vec![LineBlock {
        //     parts: vec![Parts::Text("some text".to_string())],
        //     style: Style::default().fg(Color::Red).into(),
        //     to_colorize: false,
        // }])
        assert_eq!(
            lines[0][0].style.get_style(&task, &Styles::default()),
            TextStyle::default().fg(Color::red())
        );

        let lines = Lines::from_str("[some text] and another text")?;
        assert_eq!(
            lines[0][0].parts,
            vec![Parts::Text("some text".to_string())]
        );
        assert_eq!(
            lines[0][1].parts,
            vec![Parts::Text(" and another text".to_string())]
        );

        // TODO failed
        // let lines = Lines::from_str("[some text]\\[ and escaped text \\]")?;
        // assert_eq!(lines[0][0].parts, vec![Parts::Text("some text".to_string())]);
        // assert_eq!(lines[0][1].parts, vec![Parts::Text("[ and escaped text ]".to_string())]);

        let lines = Lines::from_str("[some text]")?;
        assert_eq!(
            lines[0][0].parts,
            vec![Parts::Text("some text".to_string())]
        );

        let lines = Lines::from_str("[some text](red) text between [another text](blue, bold)")?;
        assert_eq!(
            lines[0][0].parts,
            vec![Parts::Text("some text".to_string())]
        );
        assert_eq!(
            lines[0][0].style.get_style(&task, &Styles::default()),
            TextStyle::default().fg(Color::red())
        );
        assert_eq!(
            lines[0][1].parts,
            vec![Parts::Text(" text between ".to_string())]
        );
        assert_eq!(
            lines[0][1].style.get_style(&task, &Styles::default()),
            TextStyle::default()
        );
        assert_eq!(
            lines[0][2].parts,
            vec![Parts::Text("another text".to_string())]
        );
        assert_eq!(
            lines[0][2].style.get_style(&task, &Styles::default()),
            TextStyle::default()
                .fg(Color::blue())
                .modifier(crate::config::TextModifier::Bold)
        );

        // TODO failed
        // let lines = Lines::from_str("[some text](priority:A)")?;
        // assert_eq!(
        //     lines[0][0].parts,
        //     vec![Parts::Text("some text".to_string())]
        // );
        // assert_eq!(
        //     lines[0][0].style.get_style(&task, &Styles::default()),
        //     TextStyle::default().fg(Color::red())
        // );

        let lines = Lines::from_str("some text\nnew line")?;
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0][0].parts,
            vec![Parts::Text("some text".to_string())]
        );
        assert_eq!(lines[1][0].parts,
            vec![Parts::Text("new line".to_string())]
        );

        Ok(())
    }
}
