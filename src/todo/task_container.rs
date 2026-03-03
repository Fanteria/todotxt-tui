use std::path::PathBuf;

#[derive(Clone, Debug)]
struct Hunk<T> {
    tasks: Vec<T>,
    source: PathBuf,
    edited: bool,
}

#[derive(Clone, Debug, Default)]
pub struct DataContainer<T> {
    tasks: Vec<Hunk<T>>,
}

impl<T> DataContainer<T> {
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.tasks.iter().map(|hunk| hunk.tasks.len()).sum()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.into_iter().skip(index).next()
    }
}

pub struct DataContainerIter<'a, T>(
    std::iter::FlatMap<
        std::slice::Iter<'a, Hunk<T>>,
        std::slice::Iter<'a, T>,
        fn(&'a Hunk<T>) -> std::slice::Iter<'a, T>,
    >,
);

impl<'a, T> Iterator for DataContainerIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, T> IntoIterator for &'a DataContainer<T> {
    type Item = &'a T;
    type IntoIter = DataContainerIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        // This creates an iterator over Hunks,
        // then flattens them into an iterator over Tasks.
        DataContainerIter(self.tasks.iter().flat_map(|h| h.tasks.iter()))
    }
}
