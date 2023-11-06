use crate::CONFIG;
use tui::text::Span;
use tui::widgets::ListItem;

/// Represents a list of categories, where each category is a tuple of `(&'a String, bool)`.
/// The `String` value represents name of category and the `bool` value represents
/// whether the category is selected or not.
pub struct CategoryList<'a>(pub Vec<(&'a String, bool)>);

impl<'a> CategoryList<'a> {
    /// Returns a vector of references to categories that start with the specified pattern.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The pattern to match the categories with.
    ///
    /// # Returns
    ///
    /// A vector of references to the matching categories.
    pub fn start_with(&self, pattern: &str) -> Vec<&String> {
        self.0
            .iter()
            .filter(|(item, _)| item.starts_with(pattern))
            .map(|(item, _)| *item)
            .collect()
    }

    /// Checks if the category list is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of categories in the list.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Gets the name of the category at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the category to retrieve.
    ///
    /// # Returns
    ///
    /// A reference to the name of the category.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get_name(&self, index: usize) -> &String {
        self.0[index].0
    }
}

impl<'a> From<CategoryList<'a>> for Vec<ListItem<'a>> {
    fn from(val: CategoryList<'a>) -> Self {
        val.0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        let first = String::from("first");
        let second = String::from("second");
        let third = String::from("third");
        let third2 = String::from("third2");
        let categories = CategoryList(vec![
            (&first, false),
            (&second, false),
            (&third, false),
            (&third2, false),
        ]);

        assert!(!categories.is_empty());
        assert_eq!(categories.len(), 4);
    }

    #[test]
    fn start_with() {
        let first = String::from("first");
        let second = String::from("second");
        let third = String::from("third");
        let third2 = String::from("third2");
        let categories = CategoryList(vec![
            (&first, false),
            (&second, false),
            (&third, false),
            (&third2, false),
        ]);
        assert!(categories.start_with("none").is_empty());

        let match_fi = categories.start_with("fi");
        assert_eq!(match_fi.len(), 1);
        assert_eq!(match_fi[0], &first);

        let match_fi = categories.start_with("th");
        assert_eq!(match_fi.len(), 2);
        assert_eq!(match_fi[0], &third);
        assert_eq!(match_fi[1], &third2);
    }

    #[test]
    fn create_list_of_items() {
        let first = String::from("first");
        let second = String::from("second");
        let third = String::from("third");
        let third2 = String::from("third2");
        let categories = CategoryList(vec![
            (&first, false),
            (&second, false),
            (&third, true),
            (&third2, false),
        ]);

        let items = Vec::<ListItem>::from(categories);
        assert_eq!(items.len(), 4);
        assert_eq!(items[0], ListItem::new(first.clone()));
        assert_eq!(items[1], ListItem::new(second.clone()));
        assert_eq!(
            items[2],
            ListItem::new(Span::styled(
                third.clone(),
                CONFIG.category_color.get_style()
            ))
        );
        assert_eq!(items[3], ListItem::new(third2.clone()));
    }
}
