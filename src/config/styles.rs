use super::{Styles, TextStyle};
use crate::{Result, ToDoError};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    str::FromStr,
};
use todo_txt::Task;
use tui::style::Style;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum StylesValue {
    Const(Style),
    CustomCategory,
    Priority,
}

impl StylesValue {
    pub fn get_style(&self, task: &Task, styles: &Styles) -> Style {
        use StylesValue::*;
        match self {
            Const(style) => style.to_owned(),
            CustomCategory => {
                let mut text_style = TextStyle::default();
                let mut process_projects = |prefix: &str, data: &[String]| {
                    data.iter().for_each(|category: &String| {
                        if let Some(style) = styles
                            .custom_category_style
                            .get(&(prefix.to_string() + category))
                        {
                            text_style = text_style.combine(style);
                        }
                    });
                };
                process_projects("+", task.projects());
                process_projects("@", task.contexts());
                process_projects("#", &task.hashtags);

                text_style.get_style()
            }
            Priority => styles
                .priority_style
                .get_style(task.priority.clone().into()),
        }
    }
}

impl Default for StylesValue {
    fn default() -> Self {
        Self::Const(Style::default())
    }
}

impl From<Style> for StylesValue {
    fn from(style: Style) -> Self {
        StylesValue::Const(style)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct CustomCategoryStyle(HashMap<String, TextStyle>);

impl Deref for CustomCategoryStyle {
    type Target = HashMap<String, TextStyle>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CustomCategoryStyle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromStr for CustomCategoryStyle {
    type Err = ToDoError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        fn parse(item: &str) -> Result<(String, TextStyle)> {
            let (key, value) =
                item.split_once('=')
                    .ok_or(ToDoError::CustomCategoryStyleParseFailed(
                        "Key and value must be separated by =",
                    ))?;
            Ok((key.to_string(), TextStyle::from_str(value)?))
        }
        Ok(CustomCategoryStyle(
            s.split(',').map(parse).collect::<Result<_>>()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use tui::style::Color;

    #[test]
    fn get_style() -> Result<()> {
        let task = Task::from_str("(A) Task name +project #hashtag").unwrap();
        println!("{:#?}", task);
        let styles = Styles::default();
        assert_eq!(
            Style::default(),
            styles.get_style("")?.get_style(&task, &styles)
        );
        assert!(styles.get_style("Unknown").is_err());
        assert_eq!(
            Style::default(),
            styles.get_style("hashtags")?.get_style(&task, &styles)
        );
        assert_eq!(
            Style::default().fg(Color::Red),
            styles.get_style("priority:A")?.get_style(&task, &styles)
        );
        assert_eq!(
            Style::default().fg(Color::Red),
            styles.get_style("priority")?.get_style(&task, &styles)
        );

        Ok(())
    }
}
