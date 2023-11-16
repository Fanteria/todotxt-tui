use std::collections::HashMap;

use super::{text_style::TextStyleList, Config, TextStyle};

#[derive(Default)]
pub struct Styles {
    pub priority_style: TextStyleList,
    pub projects_style: TextStyle,
    pub contexts_style: TextStyle,
    pub hashtags_style: TextStyle,
    pub category_style: TextStyle,
    pub custom_category_style: HashMap<String, TextStyle>,
}

impl Styles {
    pub fn new(config: &Config) -> Self {
        let mut styles = Styles {
            priority_style: config.priority_colors.clone(),
            category_style: config.category_style,
            projects_style: config.projects_style.combine(&config.category_style),
            contexts_style: config.contexts_style.combine(&config.category_style),
            hashtags_style: config.hashtags_style.combine(&config.category_style),
            custom_category_style: HashMap::new(),
        };
        styles.custom_category_style = config
            .custom_category_style
            .iter()
            .map(|(name, style)| {
                (
                    name.clone(),
                    style.combine(&styles.get_category_style(name)),
                )
            })
            .collect();

        styles
    }

    pub fn get_style(&self, category: &str) -> TextStyle {
        match self.custom_category_style.get(category) {
            Some(style) => *style,
            None => self.get_category_style(category),
        }
    }

    fn get_category_style(&self, category: &str) -> TextStyle {
        match category.chars().next().unwrap() {
            '+' => self.projects_style,
            '@' => self.contexts_style,
            '#' => self.hashtags_style,
            _ => self.category_style,
        }
    }
}
