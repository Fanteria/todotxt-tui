use super::{Parts, ToDo};
use crate::{
    config::{Styles, StylesValue},
    {Result, ToDoError},
};
use tui::style::Style;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct LineBlock {
    pub parts: Vec<Parts>,
    pub style: StylesValue,
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

    pub fn fill(&self, todo: &ToDo, styles: &Styles) -> Option<(String, Style)> {
        let mut ret = String::new();
        for part in &self.parts {
            ret += &part.fill(todo)?;
        }
        Some((
            ret,
            match todo.get_active() {
                Some(task) => self.style.get_style(task, styles),
                None => Style::default(),
            },
        ))
    }

    pub fn try_from_styled(value: &str, style: Option<String>, styles: &Styles) -> Result<Self> {
        Ok(LineBlock {
            parts: Self::parse_variables(value)?,
            style: match style {
                Some(style) => styles.get_style(&style)?,
                None => StylesValue::default(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
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

        assert_eq!(
            LineBlock::parse_variables("variable block not closed ${variable "),
            Err(ToDoError::ParseVariableNotClosed(String::from("variable ")))
        );
    }
}
