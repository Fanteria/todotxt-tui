use super::Parts;

use super::ToDo;
use crate::config::TextStyle;
use crate::error::ToDoError;
use crate::error::ToDoRes;
use std::str::FromStr;
use tui::style::Style;

#[derive(PartialEq, Eq, Debug)]
pub struct LineBlock {
    pub parts: Vec<Parts>,
    pub style: Style,
}

impl LineBlock {
    fn parse_variables(block: &str) -> ToDoRes<Vec<Parts>> {
        let mut ret = Vec::new();
        let mut iter = block.chars();
        let mut read_variable = false;
        let mut variable_block = false;
        let mut read = String::new();
        loop {
            let c = match iter.next() {
                Some(c) => c,
                None => break,
            };
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
                    read = String::new();
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
                todo!() // TODO error variable block not closed
            }
            Parts::from(read)
        } else {
            Parts::Text(read)
        });

        Ok(ret)
    }

    pub fn fill(&self, todo: &ToDo) -> Option<(String, Style)> {
        let mut ret = String::new();
        for part in &self.parts {
            ret += &part.fill(todo)?;
        }
        Some((ret, self.style))
    }

    pub fn try_from_styled(value: &str, style: Option<String>) -> ToDoRes<Self> {
        Ok(LineBlock {
            parts: Self::parse_variables(value)?,
            style: match style {
                Some(style) => TextStyle::from_str(&style)?.get_style(),
                None => Style::default(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_variables() -> ToDoRes<()> {
        let parts = LineBlock::parse_variables("")?;
        assert_eq!(parts[0], Parts::Text("".into()));

        let parts = LineBlock::parse_variables("some text")?;
        assert_eq!(parts[0], Parts::Text("some text".into()));

        let parts = LineBlock::parse_variables("some text $done another text")?;
        assert_eq!(parts[0], Parts::Text("some text ".into()));
        assert_eq!(parts[1], Parts::Done);
        assert_eq!(parts[2], Parts::Text("another text".into()));

        let parts = LineBlock::parse_variables("there is ${pending}x pending tasks")?;
        assert_eq!(parts[0], Parts::Text("there is ".into()));
        assert_eq!(parts[1], Parts::Pending);
        assert_eq!(parts[2], Parts::Text("x pending tasks".into()));

        let parts = LineBlock::parse_variables("special task text $some-special")?;
        assert_eq!(parts[0], Parts::Text("special task text ".into()));
        assert_eq!(parts[1], Parts::Special("some-special".into()));

        let parts = LineBlock::parse_variables("special \\$ character")?;
        assert_eq!(parts[0], Parts::Text("special $ character".into()));

        Ok(())
    }

    #[test]
    fn parse_variables_error() {
        assert_eq!(
            LineBlock::parse_variables("string with $ empty variable"),
            Err(ToDoError::EmptyVariableName(String::from(
                "string with $ empty variable"
            )))
        );

        assert_eq!(
            LineBlock::parse_variables("string with empty variable on end $"),
            Err(ToDoError::EmptyVariableName(String::from(
                "string with empty variable on end $"
            )))
        );

        assert_eq!(
            LineBlock::parse_variables("invalid escape \\"),
            Err(ToDoError::ParseBlockEscapeOnEnd(String::from(
                "invalid escape \\"
            )))
        );
    }
}
