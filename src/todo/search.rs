use crate::config::Styles;
use std::{borrow::Cow, ops::Deref};
use tui::text::Span;

pub struct Search;

pub struct SearchMatches<'a, 'b> {
    subject: Cow<'a, str>,
    to_search: Cow<'b, str>,
    act: Option<usize>,
}

pub struct SearchVisitor<'a, 'b> {
    source: &'a str,
    it: SearchMatches<'a, 'b>,
    last: Option<usize>,
}

impl Search {
    pub fn find<I, T>(vals: I, to_search: &str, to_str: impl Fn(&T) -> &str) -> Option<T>
    where
        I: Iterator<Item = T>,
    {
        for val in vals {
            let mut matches = SearchMatches::new(to_str(&val), to_search);
            if matches.next().is_some() {
                return Some(val);
            }
        }
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
                let mut visitor = SearchVisitor::new(subject, to_search);
                let mut ret = Vec::new();
                for (before_str, match_str) in visitor.by_ref() {
                    if let Some(s) = before_str {
                        ret.push(Span::styled(s, style))
                    }
                    if let Some(s) = match_str {
                        ret.push(Span::styled(s, styles.highlight.get_style()))
                    }
                }
                ret.push(Span::styled(visitor.remaining(), style));
                ret
            }
            None => vec![Span::styled(subject, style)],
        }
    }
}

impl<'a, 'b> SearchMatches<'a, 'b> {
    fn prepare_search<'x, 'y>(
        subject: &'x str,
        to_search: &'y str,
    ) -> (Cow<'x, str>, Cow<'y, str>) {
        match to_search.chars().next() {
            Some(c) if c.is_uppercase() => (subject.into(), to_search.into()),
            _ => (
                subject.to_uppercase().into(),
                to_search.to_uppercase().into(),
            ),
        }
    }

    pub fn new(source: &'a str, to_search: &'b str) -> SearchMatches<'a, 'b> {
        let (subject, to_search) = Self::prepare_search(source, to_search);
        Self {
            subject,
            to_search,
            act: None,
        }
    }
}

impl Iterator for SearchMatches<'_, '_> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        self.act = match self.act {
            Some(i) => Some(i + 1 + self.subject.get(i + 1..)?.find(self.to_search.deref())?),
            None => self.subject.find(self.to_search.deref()),
        };
        self.act
    }
}

impl<'a, 'b> SearchVisitor<'a, 'b> {
    pub fn new(source: &'a str, to_search: &'b str) -> SearchVisitor<'a, 'b> {
        Self {
            source,
            it: SearchMatches::new(source, to_search),
            last: None,
        }
    }

    pub fn act_match(&self) -> Option<&'a str> {
        match self.it.act {
            Some(act) => Some(&self.source[act..act + self.it.to_search.len()]),
            None => None,
        }
    }

    pub fn text_before(&self) -> Option<&'a str> {
        let from = self.calc_index(self.last);
        match self.it.act {
            Some(act) if from < act => Some(&self.source[from..act]),
            _ => None,
        }
    }

    pub fn remaining(&self) -> &'a str {
        &self.source[self.calc_index(self.it.act)..]
    }

    fn calc_index(&self, from: Option<usize>) -> usize {
        match from {
            Some(index) => index + self.it.to_search.len(),
            None => 0,
        }
    }
}

impl<'a> Iterator for SearchVisitor<'a, '_> {
    type Item = (Option<&'a str>, Option<&'a str>);

    fn next(&mut self) -> Option<Self::Item> {
        self.last = self.it.act;
        self.it.next()?;
        Some((self.text_before(), self.act_match()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visitor_iterator() {
        let subject = "subject to search";
        let mut it = SearchVisitor::new(subject, "search");
        assert_eq!(it.next(), Some((Some("subject to "), Some("search"))));
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);

        let subject = "In this text is lot of letters T.";
        let mut it = SearchVisitor::new(subject, "t");
        assert_eq!(it.next(), Some((Some("In "), Some("t"))));
        assert_eq!(it.next(), Some((Some("his "), Some("t"))));
        assert_eq!(it.next(), Some((Some("ex"), Some("t"))));
        assert_eq!(it.next(), Some((Some(" is lo"), Some("t"))));
        assert_eq!(it.next(), Some((Some(" of le"), Some("t"))));
        assert_eq!(it.next(), Some((None, Some("t"))));
        assert_eq!(it.next(), Some((Some("ers "), Some("T"))));
        assert_eq!(it.next(), None);
        assert_eq!(it.remaining(), ".");
    }

    #[test]
    fn find() {
        let v = [
            "line with A letter",
            "line with B letter",
            "line with C letter",
            "line with D letter",
        ];
        assert_eq!(
            *Search::find(v.iter(), "B", |s| s).unwrap(),
            "line with B letter"
        );
        assert_eq!(Search::find(v.iter(), "E", |s| s), None);
    }

    #[test]
    fn highlight() {
        let styles = Styles::default();
        let vec = Search::highlight(
            "this is contain three occurrence of 'is'",
            Some("is"),
            &styles,
            tui::prelude::Style::default(),
        );
        assert_eq!(vec.len(), 7);
        assert_eq!(vec[0].to_string(), "th");
        assert_eq!(vec[1].to_string(), "is");
        assert_eq!(vec[2].to_string(), " ");
        assert_eq!(vec[3].to_string(), "is");
        assert_eq!(vec[4].to_string(), " contain three occurrence of '");
        assert_eq!(vec[5].to_string(), "is");
        assert_eq!(vec[6].to_string(), "'");
    }
}
