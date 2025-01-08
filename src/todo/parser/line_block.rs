use super::{Parts, ToDo};
use crate::{
    config::{Styles, StylesValue},
    {Result, ToDoError},
};
use regex::Regex;
use todo_txt::Task;
use tui::style::Style;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct LineBlock {
    pub parts: Vec<Parts>,
    pub style: StylesValue,
    pub to_colorize: bool,
}

impl LineBlock {
    fn parse_variables(block: &str) -> Result<Vec<Parts>> {
        let mut ret = Vec::new();
        let mut iter = block.chars();
        let mut read_variable = false;
        let mut variable_block = false;
        let mut read = String::new();
        while let Some(c) = iter.next() {
            match c {
                '$' => {
                    read_variable = true;
                    ret.push(Parts::Text(read));
                    read = String::new();
                    match iter.next() {
                        Some('{') => variable_block = true,
                        Some(ch) if ch.is_whitespace() => {
                            return Err(ToDoError::EmptyVariableName(block.to_string()))
                        }
                        Some(ch) => read.push(ch),
                        None => return Err(ToDoError::EmptyVariableName(block.to_string())),
                    }
                }
                '}' if read_variable && variable_block => {
                    variable_block = false;
                    read_variable = false;
                    ret.push(Parts::from(read));
                    read = String::new();
                }
                c if read_variable && !variable_block && c.is_whitespace() => {
                    read_variable = false;
                    ret.push(Parts::from(read));
                    read = String::from(c);
                }
                '\\' => read.push(match iter.next() {
                    Some(ch) => ch,
                    None => return Err(ToDoError::ParseBlockEscapeOnEnd(block.to_string())),
                }),
                _ => read.push(c),
            };
        }
        ret.push(if read_variable {
            if variable_block {
                return Err(ToDoError::ParseVariableNotClosed(read));
            }
            Parts::from(read)
        } else {
            Parts::Text(read)
        });

        Ok(ret)
    }

    pub fn fill(&self, task: &Task, todo: &ToDo, styles: &Styles) -> Option<Vec<(String, Style)>> {
        let style = match todo.get_active() {
            Some(task) => self.style.get_style(task, styles),
            None => Style::default(),
        };
        let string = self
            .parts
            .iter()
            .map(|part| part.fill(task, todo))
            .collect::<Option<String>>()?;
        if !self.to_colorize {
            Some(vec![(string, style)])
        } else {
            let re = Regex::new(r"(?:^|\s)([+@#]\w+)(?:$|\s)").unwrap();
            let mut last_index = 0;
            Some(
                re.captures_iter(&string)
                    .filter_map(|m| m.get(1))
                    .flat_map(|m| {
                        let start = last_index;
                        last_index = m.end();
                        let category = string[m.start()..m.end()].to_string();
                        let category_style = styles.get_category_style(&category).get_style();
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

    pub fn try_from_styled(
        value: &str,
        style: Option<String>,
        to_colorize: bool,
        styles: &Styles,
    ) -> Result<Self> {
        Ok(LineBlock {
            parts: Self::parse_variables(value)?,
            style: match style {
                Some(style) => styles.get_style(&style)?,
                None => StylesValue::default(),
            },
            to_colorize,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::config::Conf;

    use super::*;

    #[test]
    fn parse_variables() -> Result<()> {
        let parts = LineBlock::parse_variables("")?;
        assert_eq!(parts[0], Parts::Text("".into()));

        let parts = LineBlock::parse_variables("some text")?;
        assert_eq!(parts[0], Parts::Text("some text".into()));

        let parts = LineBlock::parse_variables("some text $done another text")?;
        assert_eq!(parts[0], Parts::Text("some text ".into()));
        assert_eq!(parts[1], Parts::Done);
        assert_eq!(parts[2], Parts::Text(" another text".into()));

        let parts = LineBlock::parse_variables("there is ${pending}x pending tasks")?;
        assert_eq!(parts[0], Parts::Text("there is ".into()));
        assert_eq!(parts[1], Parts::Pending);
        assert_eq!(parts[2], Parts::Text("x pending tasks".into()));

        let parts = LineBlock::parse_variables("special task text $some-special")?;
        assert_eq!(parts[0], Parts::Text("special task text ".into()));
        assert_eq!(parts[1], Parts::Special("some-special".into()));

        let parts = LineBlock::parse_variables("special \\$ character")?;
        assert_eq!(parts[0], Parts::Text("special $ character".into()));

        let parts = LineBlock::parse_variables("Pending: $pending Done: $done")?;
        assert_eq!(parts[0], Parts::Text("Pending: ".into()));
        assert_eq!(parts[1], Parts::Pending);
        assert_eq!(parts[2], Parts::Text(" Done: ".into()));
        assert_eq!(parts[3], Parts::Done);

        Ok(())
    }

    #[test]
    fn parse_variables_error() {
        assert_eq!(
            LineBlock::parse_variables("string with $ empty variable")
                .unwrap_err()
                .to_string(),
            ToDoError::EmptyVariableName(String::from("string with $ empty variable")).to_string()
        );

        assert_eq!(
            LineBlock::parse_variables("string with empty variable on end $")
                .unwrap_err()
                .to_string(),
            ToDoError::EmptyVariableName(String::from("string with empty variable on end $"))
                .to_string()
        );

        assert_eq!(
            LineBlock::parse_variables("invalid escape \\")
                .unwrap_err()
                .to_string(),
            ToDoError::ParseBlockEscapeOnEnd(String::from("invalid escape \\")).to_string()
        );

        assert_eq!(
            LineBlock::parse_variables("variable block not closed ${variable ")
                .unwrap_err()
                .to_string(),
            ToDoError::ParseVariableNotClosed(String::from("variable ")).to_string()
        );
    }

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
fg = "Black"
        "#
            .as_bytes(),
        )
        .unwrap();

        let block = LineBlock::try_from_styled(
            "Done +project to be @splitted here #hashtag and some +custom project",
            Some(String::from("black")),
            true,
            &style,
        )?;
        assert_eq!(
            block.fill(&task, &todo, &style),
            Some(vec![
                (String::from("Done "), Style::default()),
                (
                    String::from("+project"),
                    Style::default().fg(tui::style::Color::Green)
                ),
                (String::from(" to be "), Style::default()),
                (
                    String::from("@splitted"),
                    Style::default().fg(tui::style::Color::Red)
                ),
                (String::from(" here "), Style::default()),
                (
                    String::from("#hashtag"),
                    Style::default().fg(tui::style::Color::Yellow)
                ),
                (String::from(" and some "), Style::default()),
                (
                    String::from("+custom"),
                    Style::default().fg(tui::style::Color::Black)
                ),
                (String::from(" project"), Style::default()),
            ])
        );

        Ok(())
    }
}
