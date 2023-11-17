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
                let style = style.combine(&styles.get_category_style(&name));
                (name, style)
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
