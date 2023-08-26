use super::RcCon;
use std::ops::{Deref, DerefMut};

pub struct Holder<T> {
    pub data: T,
    pub parent: RcCon,
}

impl<T> Holder<T> {
    pub fn new(data: T, parent: RcCon) -> Self {
        Self { data, parent }
    }
}

impl<T> Deref for Holder<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Holder<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
