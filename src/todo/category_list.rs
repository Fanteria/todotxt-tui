use std::collections::BTreeSet;

use super::{FilterState, ToDo, ToDoCategory};
use crate::config::Styles;
use crate::todo::search::Search;
use tui::text::Line;
use tui::widgets::ListItem;

#[derive(Debug, PartialEq, Eq)]
pub struct CategoryState<'a> {
    pub name: &'a String,
    state: Option<FilterState>,
}

/// Represents a list of categories, where each category is a tuple of `(&'a String, Option<FilterState>)`.
/// The `String` value represents name of category and the `bool` value represents
/// whether the category is selected or not.
pub struct CategoryList<'a> {
    pub vec: Vec<CategoryState<'a>>,
    styles: &'a Styles,
}

/// Represents a slice of categories, where each category is a tuple of `(&'a String, Option<FilterState>)`.
/// The `String` value represents the name of the category, and the `Option<FilterState>`
/// value represents the optional filter state associated with that category.
///
/// This struct holds a reference to an array slice of categories and a reference to styles
/// that apply to the category slice.
pub struct CategorySlice<'a> {
    vec: &'a [CategoryState<'a>],
    styles: &'a Styles,
    to_search: Option<&'a str>,
}

impl<'a> CategoryList<'a> {
    /// This function constructs a list of tasks based on the configuration
    /// in the `ToDo` struct. If the `use_done` flag is enabled, it includes
    /// both pending and completed tasks in the list. Otherwise, it includes
    /// only pending tasks.
    pub fn new(todo: &'a ToDo, category: ToDoCategory) -> Self {
        let tasks = if todo.config.use_done {
            vec![&todo.pending, &todo.done]
        } else {
            vec![&todo.pending]
        };

        let selected = todo.state.get_category(category);
        CategoryList {
            vec: tasks
                .iter()
                .flat_map(|list| list.iter())
                .flat_map(|task| category.get_data(task).iter())
                .chain(todo.state.get_category(category).keys())
                .collect::<BTreeSet<&String>>()
                .iter()
                .map(|item| CategoryState {
                    name: item,
                    state: selected.get(*item).cloned(),
                })
                .collect(),
            styles: &todo.styles,
        }
    }

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
        self.vec
            .iter()
            .filter(|s| s.name.starts_with(pattern))
            .map(|s| s.name)
            .collect()
    }

    /// Checks if the category list is empty.
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Returns the number of categories in the list.
    pub fn len(&self) -> usize {
        self.vec.len()
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
        self.vec[index].name
    }

    // TODO this is ugly as hell
    pub fn slice(&'a self, first: usize, last: usize, to_search: Option<&'a str>) -> CategorySlice {
        if last > self.vec.len() {
            CategorySlice {
                vec: &self.vec[first..],
                styles: self.styles,
                to_search,
            }
        } else {
            CategorySlice {
                vec: &self.vec[first..last],
                styles: self.styles,
                to_search,
            }
        }
    }
}

impl<'a> From<CategorySlice<'a>> for Vec<ListItem<'a>> {
    fn from(val: CategorySlice<'a>) -> Self {
        val.vec
            .iter()
            .map(|s| {
                use FilterState::*;
                ListItem::new(Line::from(Search::highlight(
                    s.name.as_str(),
                    val.to_search,
                    val.styles,
                    match s.state {
                        Some(Select) => val.styles.category_select_style.get_style(),
                        Some(Remove) => val.styles.category_remove_style.get_style(),
                        None => tui::style::Style::default(),
                    },
                )))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, str::FromStr};

    use todo_txt::Task;
    use tui::text::Span;

    use crate::config::Config;

    use super::*;

    #[test]
    fn basics() {
        let styles = Styles::default();
        let first = String::from("first");
        let second = String::from("second");
        let third = String::from("third");
        let third2 = String::from("third2");
        let categories = CategoryList {
            vec: vec![
                CategoryState {
                    name: &first,
                    state: None,
                },
                CategoryState {
                    name: &second,
                    state: None,
                },
                CategoryState {
                    name: &third,
                    state: None,
                },
                CategoryState {
                    name: &third2,
                    state: None,
                },
            ],
            styles: &styles,
        };

        assert!(!categories.is_empty());
        assert_eq!(categories.len(), 4);
    }

    #[test]
    fn start_with() {
        let styles = Styles::default();
        let first = String::from("first");
        let second = String::from("second");
        let third = String::from("third");
        let third2 = String::from("third2");
        let categories = CategoryList {
            vec: vec![
                CategoryState {
                    name: &first,
                    state: None,
                },
                CategoryState {
                    name: &second,
                    state: None,
                },
                CategoryState {
                    name: &third,
                    state: None,
                },
                CategoryState {
                    name: &third2,
                    state: None,
                },
            ],
            styles: &styles,
        };
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
        let styles = Config::default().styles;
        let first = String::from("first");
        let second = String::from("second");
        let third = String::from("third");
        let third2 = String::from("third2");
        let categories = CategoryList {
            vec: vec![
                CategoryState {
                    name: &first,
                    state: None,
                },
                CategoryState {
                    name: &second,
                    state: None,
                },
                CategoryState {
                    name: &third,
                    state: Some(FilterState::Select),
                },
                CategoryState {
                    name: &third2,
                    state: None,
                },
            ],
            styles: &styles,
        };

        let items = Vec::<ListItem>::from(categories.slice(0, 10000, None));
        assert_eq!(items.len(), 4);
        assert_eq!(items[0], ListItem::new(first.clone()));
        assert_eq!(items[1], ListItem::new(second.clone()));
        assert_eq!(
            items[2],
            ListItem::new(Span::styled(
                third.clone(),
                styles.category_select_style.get_style()
            ))
        );
        assert_eq!(items[3], ListItem::new(third2.clone()));
    }

    #[test]
    fn test_categeries_list() -> Result<(), Box<dyn Error>> {
        fn create_vec(items: &[String]) -> Vec<CategoryState> {
            let mut vec = Vec::new();
            items.iter().for_each(|name| {
                vec.push(CategoryState { name, state: None });
            });
            vec
        }

        let mut todo = ToDo::default();

        todo.add_task(Task::from_str("x 1 +project1 @context1 #hashtag1").unwrap());
        todo.add_task(Task::from_str("2 +project2 @context2").unwrap());
        todo.add_task(Task::from_str("3 +project3 @context3").unwrap());
        todo.add_task(Task::from_str("4 +project2 @context3 #hashtag1").unwrap());
        todo.add_task(Task::from_str("5 +project3 @context3 #hashtag2").unwrap());
        todo.add_task(Task::from_str("6 +project3 @context2 #hashtag2").unwrap());

        assert_eq!(
            todo.get_categories(ToDoCategory::Projects).vec,
            create_vec(&[String::from("project2"), String::from("project3")])
        );
        assert_eq!(
            todo.get_categories(ToDoCategory::Contexts).vec,
            create_vec(&[String::from("context2"), String::from("context3")])
        );
        assert_eq!(
            todo.get_categories(ToDoCategory::Hashtags).vec,
            create_vec(&[String::from("hashtag1"), String::from("hashtag2")])
        );

        todo.config.use_done = true;
        assert_eq!(
            todo.get_categories(ToDoCategory::Projects).vec,
            create_vec(&[
                String::from("project1"),
                String::from("project2"),
                String::from("project3"),
            ])
        );
        assert_eq!(
            todo.get_categories(ToDoCategory::Contexts).vec,
            create_vec(&[
                String::from("context1"),
                String::from("context2"),
                String::from("context3"),
            ])
        );
        assert_eq!(
            todo.get_categories(ToDoCategory::Hashtags).vec,
            create_vec(&[String::from("hashtag1"), String::from("hashtag2")])
        );

        Ok(())
    }
}
