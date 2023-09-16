use super::RcCon;
use std::ops::{Deref, DerefMut};

/// A generic data holder with a reference to its parent container.
pub struct Holder<T> {
    pub data: T,
    pub parent: RcCon,
}

impl<T> Holder<T> {
    /// Creates a new `Holder` instance with the specified data
    /// and parent container reference.
    ///
    /// # Parameters
    ///
    /// - `data`: The data of generic type `T` to be stored in the holder.
    /// - `parent`: A reference to the parent container, represented
    ///             as an RcCon (reference-counted) reference.
    ///
    /// # Returns
    ///
    /// A new `Holder` instance containing the provided data and parent reference.
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
