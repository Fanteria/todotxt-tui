use super::super::Widget;
use super::Holder;
use super::RcCon;
use crate::error::{ErrorToDo, ErrorType, ToDoRes};

use super::super::render_trait::Render;
use tui::{backend::Backend, layout::Rect, Frame};

pub enum IItem {
    Container(RcCon),
    Widget(Holder<Widget>),
}

pub enum Item {
    Container(RcCon),
    Widget(Widget),
}

impl IItem {
    pub fn new(item: Item, parent: RcCon) -> Self {
        match item {
            Item::Widget(w) => Self::Widget(Holder::new(w, parent)),
            Item::Container(c) => {
                c.borrow_mut().parent = Some(parent);
                Self::Container(c)
            }
        }
    }

    #[allow(dead_code)]
    pub fn actual(&self) -> ToDoRes<&Widget> {
        match self {
            Self::Widget(w) => Ok(w),
            Self::Container(_) => Err(ErrorToDo::new(
                ErrorType::ActiveIsNotWidget,
                "Invalid state, active container is not widget.",
            )),
        }
    }

    pub fn actual_mut(&mut self) -> ToDoRes<&mut Widget> {
        match self {
            Self::Widget(w) => Ok(w),
            Self::Container(_) => Err(ErrorToDo::new(
                ErrorType::ActiveIsNotWidget,
                "Invalid state, active container is not widget.",
            )),
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
