use crate::CONFIG;
use tui::text::Span;
use tui::widgets::ListItem;

pub struct CategoryList<'a>(pub Vec<(&'a String, bool)>);

impl<'a> CategoryList<'a> {
    pub fn start_with(&self, pattern: &str) -> Vec<&String> {
        self.0
            .iter()
            .filter(|(item, _)| item.starts_with(pattern))
            .map(|(item, _)| *item)
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_name(&self, index: usize) -> &String {
        self.0[index].0
    }
}

impl<'a> Into<Vec<ListItem<'a>>> for CategoryList<'a> {
    fn into(self) -> Vec<ListItem<'a>> {
        self.0
            .iter()
            .map(|category| {
                if category.1 {
                    ListItem::new(Span::styled(
                        category.0.clone(),
                        CONFIG.category_color.get_style(),
                    ))
                } else {
                    ListItem::new(category.0.clone())
                }
            })
            .collect()
    }
}
