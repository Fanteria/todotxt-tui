mod line;
mod line_block;
mod parts;

use super::{ToDo, ToDoData};
use crate::{config::Styles, ToDoError, ToDoRes};
use line::Line;
use line_block::LineBlock;
use parts::Parts;
use std::iter::Peekable;
use tui::style::Style;

pub struct Parser {
    lines: Vec<Line>,
    styles: Styles,
}

impl Parser {
    pub fn new(value: &str, styles: Styles) -> ToDoRes<Self> {
        let lines = Parser::parse(value, &styles)?;
        log::debug!("Loaded parser: {:#?}", lines);
        Ok(Parser { lines, styles })
    }

    fn read_block(iter: &mut Peekable<std::str::Chars<'_>>, delimiter: char) -> ToDoRes<String> {
        let mut read = String::default();
        loop {
            let c = match iter.next() {
                Some(c) => c,
                None => return Err(ToDoError::ParseBlockNotClosed(read.to_string())),
            };
            match c {
                '\\' => read.push(match iter.next() {
                    Some(ch) => ch,
                    None => return Err(ToDoError::ParseBlockEscapeOnEnd(read + "\\")),
                }),
                c if c == delimiter => break,
                _ => read.push(c),
            };
        }
        Ok(read)
    }

    fn parse(template: &str, styles: &Styles) -> ToDoRes<Vec<Line>> {
        let mut ret = Vec::new();
        let mut line = Line::default();
        let mut act = String::default();
        let mut iter = template.chars().peekable();
        while let Some(c) = iter.next() {
            match c {
                '[' => {
                    let block = Parser::read_block(&mut iter, ']')?;
                    line.add_span_styled(&act, None, styles)?;
                    act = String::default();
                    let mut style = None;
                    if Some(&'(') == iter.peek() {
                        iter.next();
                        style = Some(Parser::read_block(&mut iter, ')')?);
                    }
                    line.add_span_styled(&block, style, styles)?;
                }
                '\\' => act.push(match iter.next() {
                    Some(ch) => ch,
                    None => return Err(ToDoError::ParseBlockEscapeOnEnd(act + "\\")),
                }),
                '\n' => {
                    line.add_span_styled(&act, None, styles)?;
                    act = String::default();
                    ret.push(line);
                    line = Line::default();
                }
                _ => act.push(c),
            }
        }
        line.add_span_styled(&act, None, styles)?;
        ret.push(line);
        Ok(ret)
    }

    pub fn fill(&self, todo: &ToDo) -> Vec<Vec<(String, Style)>> {
        self.lines
            .iter()
            .filter_map(|line| line.fill(todo, &self.styles))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Line;
    use super::*;
    use tui::style::Color;
    use tui::style::Modifier;

    #[test]
    fn read_block() -> ToDoRes<()> {
        let mut iter = "block to parse]".chars().peekable();
        assert_eq!(&Parser::read_block(&mut iter, ']')?, "block to parse");
        assert_eq!(&iter.collect::<String>(), "");

        let mut iter = "Some style block)".chars().peekable();
        assert_eq!(&Parser::read_block(&mut iter, ')')?, "Some style block");
        assert_eq!(&iter.collect::<String>(), "");

        let mut iter = "block to parse] some other text".chars().peekable();
        assert_eq!(&Parser::read_block(&mut iter, ']')?, "block to parse");
        assert_eq!(&iter.collect::<String>(), " some other text");

        let mut iter = "block to parse \\] with some \\\\ escapes]"
            .chars()
            .peekable();
        assert_eq!(
            &Parser::read_block(&mut iter, ']')?,
            "block to parse ] with some \\ escapes"
        );
        assert_eq!(&iter.collect::<String>(), "");

        Ok(())
    }

    #[test]
    fn read_block_error() {
        let mut iter = "not closed block".chars().peekable();
        assert_eq!(
            Parser::read_block(&mut iter, ']'),
            Err(ToDoError::ParseBlockNotClosed(
                "not closed block".to_string()
            ))
        );

        let mut iter = "not closed block \\".chars().peekable();
        assert_eq!(
            Parser::read_block(&mut iter, ']'),
            Err(ToDoError::ParseBlockEscapeOnEnd(
                "not closed block \\".to_string()
            ))
        );
    }

    #[test]
    fn parse() -> ToDoRes<()> {
        let styles = Styles::default();
        assert_eq!(Parser::parse("", &Styles::default())?[0], Line::default());
        assert_eq!(
            Parser::parse("some text", &Styles::default())?[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text".to_string())],
                style: styles.get_style_default(),
            }])
        );
        assert_eq!(
            Parser::parse("some text \\[ with escapes \\]", &Styles::default())?[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text [ with escapes ]".to_string())],
                style: styles.get_style_default(),
            }])
        );
        assert_eq!(
            Parser::parse("[some text](Red)", &Styles::default())?[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text".to_string())],
                style: styles.get_style_from_style(Style::default().fg(Color::Red)),
            }])
        );
        assert_eq!(
            Parser::parse("[some text] and another text", &Styles::default())?[0],
            Line(vec![
                LineBlock {
                    parts: vec![Parts::Text("some text".to_string())],
                    style: styles.get_style_default(),
                },
                LineBlock {
                    parts: vec![Parts::Text(" and another text".to_string())],
                    style: styles.get_style_default(),
                }
            ])
        );
        assert_eq!(
            Parser::parse("[some text]\\[ and escaped text \\]", &Styles::default())?[0],
            Line(vec![
                LineBlock {
                    parts: vec![Parts::Text("some text".to_string())],
                    style: styles.get_style_default(),
                },
                LineBlock {
                    parts: vec![Parts::Text("[ and escaped text ]".to_string())],
                    style: styles.get_style_default(),
                }
            ])
        );
        assert_eq!(
            Parser::parse("[some text]", &Styles::default())?[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text".to_string())],
                style: styles.get_style_default(),
            }])
        );
        assert_eq!(
            Parser::parse(
                "[some text](red) text between [another text](blue bold)",
                &Styles::default()
            )?[0],
            Line(vec![
                LineBlock {
                    parts: vec![Parts::Text("some text".to_string())],
                    style: styles.get_style_from_style(Style::default().fg(Color::Red)),
                },
                LineBlock {
                    parts: vec![Parts::Text(" text between ".to_string())],
                    style: styles.get_style_default(),
                },
                LineBlock {
                    parts: vec![Parts::Text("another text".to_string())],
                    style: styles.get_style_from_style(
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD)
                    ),
                }
            ])
        );
        assert_eq!(
            Parser::parse("[some text](priority:A)", &Styles::default())?[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text".to_string())],
                style: styles.get_style_from_style(Style::default().fg(Color::Red)),
            },])
        );
        let parse = Parser::parse("some text\nnew line", &Styles::default())?;
        assert_eq!(parse.len(), 2);
        assert_eq!(
            parse[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text".to_string())],
                style: styles.get_style_default(),
            }])
        );
        assert_eq!(
            parse[1],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("new line".to_string())],
                style: styles.get_style_default(),
            }])
        );

        Ok(())
    }

    #[test]
    fn parse_error() {
        assert_eq!(
            Parser::parse("escape on end of line \\", &Styles::default()),
            Err(ToDoError::ParseBlockEscapeOnEnd(
                "escape on end of line \\".to_string()
            ))
        );
    }

    #[test]
    fn fill_base() -> ToDoRes<()> {
        let parser = Parser::new("some text", Styles::default())?;
        let mut todo = ToDo::default();
        todo.new_task("task").unwrap();
        todo.new_task("x done task").unwrap();

        assert_eq!(parser.fill(&todo), Vec::<Vec<(String, Style)>>::new());

        todo.set_active(ToDoData::Pending, 0);
        assert_eq!(
            parser.fill(&todo),
            vec![vec![(String::from("some text"), Style::default())]]
        );

        Ok(())
    }

    #[test]
    fn fill_counts() -> ToDoRes<()> {
        let parser = Parser::new("Done: $done Pending: $pending", Styles::default())?;
        let mut todo = ToDo::default();
        todo.new_task("task").unwrap();
        todo.new_task("x done task").unwrap();
        todo.set_active(ToDoData::Pending, 0);

        assert_eq!(
            parser.fill(&todo),
            vec![vec![(String::from("Done: 1 Pending: 1"), Style::default())]]
        );

        Ok(())
    }
}
