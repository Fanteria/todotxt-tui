use tui::text::Span;

use crate::config::Styles;

pub struct Search;

impl Search {
    // TODO
    pub fn find<'a, 'b, I, T>(vals: I, to_search: &str) -> Option<&'a T>
    where
        I: Iterator<Item = &'a T>,
        T: Into<&'b str>,
    {

        // vals.min()
        None
    }

    pub fn highlight<'a>(
        subject: &'a str,
        to_search: Option<&str>,
        styles: &'a Styles,
        style: tui::prelude::Style,
    ) -> Vec<Span<'a>> {
        match to_search {
            Some(to_search) => {
                let (search_subject, to_search) = match to_search.chars().next() {
                    Some(c) if c.is_uppercase() => (subject.to_string(), String::from(to_search)),
                    _ => (subject.to_uppercase(), to_search.to_uppercase()),
                };
                log::debug!("Search '{to_search}' in '{search_subject}', orig: {subject}");
                match search_subject.find(&to_search) {
                    // TODO this match only first pattern
                    Some(i) => vec![
                        Span::styled(&subject[..i], style),
                        Span::styled(
                            &subject[i..i + to_search.len()],
                            styles.highlight.get_style(),
                        ),
                        Span::styled(&subject[i + to_search.len()..], style),
                    ],
                    None => vec![Span::styled(subject, style)],
                }
            }
            None => return vec![Span::styled(subject, style)],
        }
    }
}
