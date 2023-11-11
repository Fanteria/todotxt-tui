use std::str::FromStr;

use super::{widget_base::WidgetBase, widget_trait::State};
use crate::{
    config::TextStyle,
    error::{ToDoError, ToDoRes},
    todo::{ToDo, ToDoData},
    ui::UIEvent,
    CONFIG,
};
use chrono::NaiveDate;
use tui::{
    backend::Backend,
    style::Style,
    widgets::{Paragraph, Wrap},
    Frame,
};

#[derive(Debug, PartialEq, Eq)]
enum Parts {
    Text(String),
    Pending,
    Done,
    Subject,
    Priority,
    CreateDate,
    FinishDate,
    Finished,
    TresholdDate,
    DueDate,
    Contexts,
    Projects,
    Hashtags,
    Special(String),
}

impl Parts {
    fn fill(&self, todo: &ToDo) -> Option<String> {
        use Parts::*;
        let process_vec = |vec: &[String]| {
            if vec.is_empty() {
                None
            } else {
                Some(vec.join(", "))
            }
        };
        match todo.get_active() {
            Some(task) => match self {
                Text(text) => Some(text.to_string()),
                Pending => Some(todo.len(ToDoData::Pending).to_string()),
                Done => Some(todo.len(ToDoData::Done).to_string()),
                Subject => Some(task.subject.clone()),
                Priority => Some(task.priority.to_string()),
                CreateDate => task.create_date.map(|d| d.to_string()),
                FinishDate => task.finish_date.map(|d| d.to_string()),
                Finished => Some(task.finished.to_string()),
                TresholdDate => task.threshold_date.map(|d| d.to_string()),
                DueDate => task.due_date.map(|d| d.to_string()),
                Contexts => process_vec(task.contexts()),
                Projects => process_vec(task.projects()),
                Hashtags => process_vec(&task.hashtags),
                Special(special) => task.tags.get(special).cloned(),
            },
            None => None,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct LineBlock {
    parts: Vec<Parts>,
    style: Style,
}

impl LineBlock {
    fn fill(&self, todo: &ToDo) -> Option<(String, Style)> {
        let mut ret = String::new();
        for part in &self.parts {
            ret += &part.fill(todo)?;
        }
        Some((ret, self.style))
    }

    fn try_from_styled(value: &str, style: Option<String>) -> ToDoRes<Self> {
        Ok(LineBlock {
            parts: StatePreview::parse_variables(value)?,
            style: match style {
                Some(style) => TextStyle::from_str(&style)?.get_style(),
                None => Style::default(),
            },
        })
    }
}

impl TryFrom<&str> for LineBlock {
    type Error = ToDoError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_styled(value, None)
    }
}

#[derive(Default, PartialEq, Eq, Debug)]
struct Line(Vec<LineBlock>);
impl Line {
    fn add_span(&mut self, parts: &str) -> ToDoRes<()> {
        self.add_span_styled(parts, None)
    }

    fn add_span_styled(&mut self, parts: &str, style: Option<String>) -> ToDoRes<()> {
        if !parts.is_empty() {
            self.0.push(LineBlock::try_from_styled(parts, style)?);
        }
        Ok(())
    }

    fn fill(&self, todo: &ToDo) -> Vec<(String, Style)> {
        self.0.iter().map_while(|block| block.fill(todo)).collect()
    }
}

impl From<String> for Parts {
    fn from(value: String) -> Self {
        use Parts::*;
        match value.to_lowercase().as_str() {
            "pending" => Pending,
            "done" => Done,
            "subject" => Subject,
            "priority" => Priority,
            "createDate" => CreateDate,
            "finishDate" => FinishDate,
            "finished" => Finished,
            "treshold_date" => TresholdDate,
            "dueDate" => DueDate,
            "contexts" => Contexts,
            "projects" => Projects,
            "hashtags" => Hashtags,
            _ => Special(value),
        }
    }
}

/// Represents the state for a preview widget that displays task details.
pub struct StatePreview {
    base: WidgetBase,
    format: String,
}

impl StatePreview {
    /// Creates a new `StatePreview` instance.
    ///
    /// # Parameters
    ///
    /// - `base`: The base properties shared among different widget types.
    /// - `format`: The format string used to generate the content for the preview.
    ///
    /// # Returns
    ///
    /// A new `StatePreview` instance.
    pub fn new(base: WidgetBase, format: String) -> Self {
        StatePreview { format, base }
    }

    /// Generates the content for the preview based on the current task data.
    ///
    /// # Returns
    ///
    /// A string containing the formatted task details.
    fn get_content(&self) -> String {
        let data = self.base.data();
        let task = match data.get_active() {
            Some(s) => s,
            None => return String::from(""),
        };
        let date_to_str = |date: Option<NaiveDate>| match date {
            Some(date) => date.to_string(),
            None => String::from(""),
        };
        // TODO remove
        let mut content = self.format.clone();
        // let parsed = StatePreview::parse(&self.format);
        // println!("{:?}", parsed);
        // -----
        task.tags.iter().for_each(|(key, value)| {
            content = content.replace(&("{".to_string() + key + "}"), value);
        });
        content
            .replace("{n}", &data.len(ToDoData::Pending).to_string())
            .replace("{N}", &data.len(ToDoData::Done).to_string())
            .replace("{s}", &task.subject)
            .replace("{p}", &task.priority.to_string())
            .replace("{c}", &date_to_str(task.create_date))
            .replace("{f}", &date_to_str(task.finish_date))
            .replace("{F}", &task.finished.to_string())
            .replace("{t}", &date_to_str(task.threshold_date))
            .replace("{d}", &date_to_str(task.due_date))
            .replace("{C}", &task.contexts().join(", "))
            .replace("{P}", &task.projects().join(", "))
            .replace("{H}", &task.hashtags.join(", "))
    }

    fn read_block(iter: &mut std::str::Chars<'_>, delimiter: char) -> String {
        let mut read = String::default();
        loop {
            let c = match iter.next() {
                Some(c) => c,
                None => break, // TODO errror, block is not closed
            };
            match c {
                '\\' => read.push(iter.next().unwrap()),
                c if c == delimiter => break,
                _ => read.push(c),
            };
        }
        read
    }

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
                        Some(ch) => read.push(ch),
                        None => {} // TODO error, empty variable name
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
                '\\' => read.push(iter.next().unwrap()),
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

    fn parse(template: &str) -> ToDoRes<Vec<Line>> {
        let mut ret = Vec::new();
        let mut line = Line::default();
        let mut act = String::default();
        let mut iter = template.chars().into_iter();
        loop {
            let c = match iter.next() {
                Some(c) => c,
                None => break,
            };
            match c {
                '[' => {
                    let block = StatePreview::read_block(&mut iter, ']');
                    line.add_span(&act)?;
                    act = String::default();
                    let mut style = None;
                    match iter.next() {
                        Some('(') => style = Some(StatePreview::read_block(&mut iter, ')')),
                        Some('\\') => act.push(iter.next().unwrap()),
                        Some(ch) => act.push(ch),
                        None => {
                            act = block;
                            break;
                        }
                    }
                    line.add_span_styled(&block, style)?;
                }
                '\\' => act.push(iter.next().unwrap()),
                '\n' => {
                    line.add_span(&act)?;
                    act = String::default();
                    ret.push(line);
                    line = Line::default();
                }
                _ => act.push(c),
            }
        }
        line.add_span(&act)?;
        ret.push(line);
        Ok(ret)
    }
}

impl State for StatePreview {
    fn handle_event_state(&mut self, _: UIEvent) -> bool {
        false
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>) {
        let b = StatePreview::parse(&self.format).unwrap();
        let lines = b.iter().map(|line| line.fill(&self.base.data())).collect::<Vec<_>>();
        let mut paragraph = Paragraph::new(
            lines
                .iter()
                .map(|line| {
                    let mut l = tui::text::Line::default();
                    l.spans = line
                        .iter()
                        .map(|(text, style)| tui::text::Span::styled(text, *style))
                        .collect::<Vec<_>>();
                    l
                })
                .collect::<Vec<_>>(),
        )
        .block(self.get_block());

        if CONFIG.wrap_preview {
            paragraph = paragraph.wrap(Wrap { trim: true })
        }
        f.render_widget(paragraph, self.base.chunk);
    }

    fn get_base(&self) -> &WidgetBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut WidgetBase {
        &mut self.base
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use tui::style::Color;
    use tui::style::Modifier;

    #[test]
    fn read_block() {
        let mut iter = "block to parse]".chars();
        assert_eq!(&StatePreview::read_block(&mut iter, ']'), "block to parse");
        assert_eq!(&iter.collect::<String>(), "");

        let mut iter = "Some style block)".chars();
        assert_eq!(
            &StatePreview::read_block(&mut iter, ')'),
            "Some style block"
        );
        assert_eq!(&iter.collect::<String>(), "");

        let mut iter = "block to parse] some other text".chars();
        assert_eq!(&StatePreview::read_block(&mut iter, ']'), "block to parse");
        assert_eq!(&iter.collect::<String>(), " some other text");

        let mut iter = "block to parse \\] with some \\\\ escapes]".chars();
        assert_eq!(
            &StatePreview::read_block(&mut iter, ']'),
            "block to parse ] with some \\ escapes"
        );
        assert_eq!(&iter.collect::<String>(), "");
    }

    #[test]
    fn parse_variables() -> ToDoRes<()> {
        let parts = StatePreview::parse_variables("")?;
        assert_eq!(parts[0], Parts::Text("".into()));

        let parts = StatePreview::parse_variables("some text")?;
        assert_eq!(parts[0], Parts::Text("some text".into()));

        let parts = StatePreview::parse_variables("some text $done another text")?;
        assert_eq!(parts[0], Parts::Text("some text ".into()));
        assert_eq!(parts[1], Parts::Done);
        assert_eq!(parts[2], Parts::Text("another text".into()));

        let parts = StatePreview::parse_variables("there is ${pending}x pending tasks")?;
        assert_eq!(parts[0], Parts::Text("there is ".into()));
        assert_eq!(parts[1], Parts::Pending);
        assert_eq!(parts[2], Parts::Text("x pending tasks".into()));

        let parts = StatePreview::parse_variables("spacial task text $some-special")?;
        assert_eq!(parts[0], Parts::Text("spacial task text ".into()));
        assert_eq!(parts[1], Parts::Special("some-special".into()));

        Ok(())
    }

    #[test]
    fn parse() -> ToDoRes<()> {
        assert_eq!(StatePreview::parse("")?[0], Line::default());
        assert_eq!(
            StatePreview::parse("some text")?[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text".to_string())],
                style: Style::default()
            }])
        );
        assert_eq!(
            StatePreview::parse("some text \\[ with escapes \\]")?[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text [ with escapes ]".to_string())],
                style: Style::default()
            }])
        );
        assert_eq!(
            StatePreview::parse("[some text](red)")?[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text".to_string())],
                style: Style::default().fg(Color::Red)
            }])
        );
        assert_eq!(
            StatePreview::parse("[some text] and another text")?[0],
            Line(vec![
                LineBlock {
                    parts: vec![Parts::Text("some text".to_string())],
                    style: Style::default()
                },
                LineBlock {
                    parts: vec![Parts::Text(" and another text".to_string())],
                    style: Style::default()
                }
            ])
        );
        assert_eq!(
            StatePreview::parse("[some text]\\[ and escaped text \\]")?[0],
            Line(vec![
                LineBlock {
                    parts: vec![Parts::Text("some text".to_string())],
                    style: Style::default()
                },
                LineBlock {
                    parts: vec![Parts::Text("[ and escaped text ]".to_string())],
                    style: Style::default()
                }
            ])
        );
        assert_eq!(
            StatePreview::parse("[some text]")?[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text".to_string())],
                style: Style::default()
            }])
        );
        assert_eq!(
            StatePreview::parse("[some text](red) text between [another text](blue bold)")?[0],
            Line(vec![
                LineBlock {
                    parts: vec![Parts::Text("some text".to_string())],
                    style: Style::default().fg(Color::Red)
                },
                LineBlock {
                    parts: vec![Parts::Text(" text between ".to_string())],
                    style: Style::default()
                },
                LineBlock {
                    parts: vec![Parts::Text("another text".to_string())],
                    style: Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                }
            ])
        );
        let parse = StatePreview::parse("some text\nnew line")?;
        assert_eq!(parse.len(), 2);
        assert_eq!(
            parse[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text".to_string())],
                style: Style::default()
            }])
        );
        assert_eq!(
            parse[1],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("new line".to_string())],
                style: Style::default()
            }])
        );

        Ok(())
    }
}
