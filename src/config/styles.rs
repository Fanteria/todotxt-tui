use std::{collections::HashMap, str::FromStr};

use super::{text_style::TextStyleList, Config, TextStyle};
use todo_txt::Task;
use tui::style::Style;

use crate::error::ToDoRes;

#[derive(Default)]
pub struct Styles {
    pub priority_style: TextStyleList,
    pub projects_style: TextStyle,
    pub contexts_style: TextStyle,
    pub hashtags_style: TextStyle,
    pub category_style: TextStyle,
    pub custom_category_style: HashMap<String, TextStyle>,
}

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
                        if let Some(style) =
                            styles.custom_category_style.get(&(prefix.to_string() + category))
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
            Priority => styles.priority_style.get_style(task.priority.clone().into()),
        }
    }
}

impl Styles {
    pub fn new(config: &Config) -> Self {
        let category_style = config.get_category_style();
        let mut styles = Styles {
            priority_style: config.get_priority_colors(),
            category_style,
            projects_style: config.get_projects_style().combine(&category_style),
            contexts_style: config.get_contexts_style().combine(&category_style),
            hashtags_style: config.get_hashtags_style().combine(&category_style),
            custom_category_style: HashMap::new(),
        };
        styles.custom_category_style = config
            .get_custom_category_style()
            .into_iter()
            .map(|(name, style)| {
                let style = style.combine(&styles.get_category_base_style(&name));
                (name, style)
            })
            .collect();
        styles
    }

    pub fn get_style_default(&self) -> StylesValue {
        StylesValue::Const(Style::default())
    }

    pub fn get_style_from_style(&self, style: Style) -> StylesValue {
        StylesValue::Const(style)
    }

    pub fn get_style(&self, name: &str) -> ToDoRes<StylesValue> {
        println!("{}", name);
        use StylesValue::*;
        Ok(match name {
            "priority" => Priority,
            "custom_category" => CustomCategory,
            "projects" => Const(self.projects_style.get_style()),
            "contexts" => Const(self.contexts_style.get_style()),
            "hashtags" => Const(self.hashtags_style.get_style()),
            "category" => Const(self.category_style.get_style()),
            _ => {
                if name.starts_with("priority:") {
                    if let Some(priority) = name.get("priority:".len()..) {
                        return Ok(Const(
                            match self
                                .priority_style
                                .get_style_from_str(&priority.to_uppercase())
                            {
                                Some(style) => style.get_style(),
                                None => Style::default(),
                            }),
                        );
                    }
                } else if name.starts_with("custom_category:") {
                    if let Some(custom_category) = name.get("custom_category:".len()..) {
                        if let Some(custom_category) =
                            self.custom_category_style.get(custom_category)
                        {
                            return Ok(Const(custom_category.get_style()));
                        }
                    }
                }
                Const(TextStyle::from_str(name)?.get_style()) // TODO do not ignore error
            }
        })
    }

    pub fn get_category_style(&self, category: &str) -> TextStyle {
        match self.custom_category_style.get(category) {
            Some(style) => *style,
            None => self.get_category_base_style(category),
        }
    }

    fn get_category_base_style(&self, category: &str) -> TextStyle {
        match category.chars().next().unwrap() {
            '+' => self.projects_style,
            '@' => self.contexts_style,
            '#' => self.hashtags_style,
            _ => self.category_style,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use tui::style::Color;

    #[test]
    fn get_style() -> ToDoRes<()> {
        let task = Task::from_str("(A) Task name +project #hashtag").unwrap();
        println!("{:#?}", task);
        let styles = Styles::new(&Config::default());
        assert_eq!(Style::default(), styles.get_style("")?.get_style(&task, &styles));
        assert!(
            styles.get_style("Unknown").is_err()
        );
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
