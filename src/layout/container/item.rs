use super::super::Widget;
use super::Holder;
use super::RcCon;
use crate::error::{ToDoError, ToDoRes};

use super::super::render_trait::Render;
use tui::{backend::Backend, layout::Rect, Frame};

/// Enum representing an item within a container,
/// which can either be a container itself or a widget.
pub enum IItem {
    Container(RcCon),
    Widget(Holder<Widget>),
}

/// Enum representing an item within a container,
/// which can either be a container itself or a widget.
pub enum Item {
    Container(RcCon),
    Widget(Widget),
}

impl IItem {
    /// Creates a new `IItem` instance based on the provided `Item` and parent container reference.
    ///
    /// # Parameters
    ///
    /// - `item`: The `Item` to be converted into an `IItem`.
    /// - `parent`: A reference to the parent container represented as an `RcCon` (reference-counted
    ///   container reference).
    ///
    /// # Returns
    ///
    /// A new `IItem` instance containing the item and parent reference.
    pub fn new(item: Item, parent: RcCon) -> Self {
        match item {
            Item::Widget(w) => Self::Widget(Holder::new(w, parent)),
            Item::Container(c) => {
                c.borrow_mut().parent = Some(parent);
                Self::Container(c)
            }
        }
    }

    /// Retrieves an immutable reference to the contained `Widget` if the item is a `Widget`, or returns
    /// an error if it is a `Container`.
    ///
    /// # Returns
    ///
    /// - `Ok(&Widget)`: An immutable reference to the contained `Widget`.
    /// - `Err(ToDoError)`: An error indicating that the item is not a `Widget`.
    pub fn actual(&self) -> ToDoRes<&Widget> {
        match self {
            Self::Widget(w) => Ok(w),
            Self::Container(_) => Err(ToDoError::ActiveIsNotWidget),
        }
    }

    /// Retrieves a mutable reference to the contained `Widget` if the item is a `Widget`, or returns
    /// an error if it is a `Container`.
    ///
    /// # Returns
    ///
    /// - `Ok(&mut Widget)`: A mutable reference to the contained `Widget`.
    /// - `Err(ToDoError)`: An error indicating that the item is not a `Widget`.
    pub fn actual_mut(&mut self) -> ToDoRes<&mut Widget> {
        match self {
            Self::Widget(w) => Ok(w),
            Self::Container(_) => Err(ToDoError::ActiveIsNotWidget),
        }
    }
}

impl Render for IItem {
    fn render<B: Backend>(&self, f: &mut Frame<B>) {
        match self {
            IItem::Widget(w) => w.render(f),
            IItem::Container(container) => container.borrow().render(f),
        }
    }

    fn focus(&mut self) {
        if let IItem::Widget(w) = self {
            w.data.focus();
        }
    }

    fn unfocus(&mut self) {
        if let IItem::Widget(w) = self {
            w.data.unfocus();
        }
    }

    fn update_chunk(&mut self, chunk: Rect) {
        match self {
            IItem::Widget(w) => w.update_chunk(chunk),
            IItem::Container(container) => container.borrow_mut().update_chunk(chunk),
        }
    }
}

