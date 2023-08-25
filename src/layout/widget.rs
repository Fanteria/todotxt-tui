mod state_categories;
mod state_list;
mod state_preview;
mod widget_base;
mod widget_list;
// mod widget_state;
pub mod widget_trait;
pub mod widget_type;

use crate::layout::widget::widget_base::WidgetBase;
use crate::todo::{ToDo, ToDoCategory, ToDoData};
use crate::CONFIG;
use state_categories::StateCategories;
use state_list::StateList;
use state_preview::StatePreview;
use std::sync::MutexGuard;
use std::sync::{Arc, Mutex};
use tui::widgets::Block;
use tui::{backend::Backend, prelude::Rect, Frame};
use widget_trait::State;
use widget_type::WidgetType;

use crossterm::event::KeyEvent;

pub type RCToDo = Arc<Mutex<ToDo>>;

#[enum_dispatch(State)]
pub enum Widget {
    List(StateList),
    Category(StateCategories),
    Preview(StatePreview),
}

impl Widget {
    pub fn new(widget_type: WidgetType, data: RCToDo) -> Self {
        use WidgetType::*;
        let base = WidgetBase::new(widget_type.to_string(), data);
        match widget_type {
            List => Self::List(StateList::new(
                base,
                ToDoData::Pending,
                CONFIG
                    .list_active_color
                    .combine(&CONFIG.pending_active_color)
                    .get_style(),
                CONFIG.list_shift,
                CONFIG.pending_sort,
            )),
            Done => Self::List(StateList::new(
                base,
                ToDoData::Done,
                CONFIG
                    .list_active_color
                    .combine(&CONFIG.done_active_color)
                    .get_style(),
                CONFIG.list_shift,
                CONFIG.done_sort,
            )),
            Project => Self::Category(StateCategories::new(base, ToDoCategory::Projects)),
            Context => Self::Category(StateCategories::new(base, ToDoCategory::Contexts)),
            Hashtag => Self::Category(StateCategories::new(base, ToDoCategory::Hashtags)),
            Preview => Self::Preview(StatePreview::new(
                base,
                "Pending: {n}   Done: {N}\nSubject: {s}\nPriority: {p}\nCreate date: {c}",
            )),
        }
    }

    pub fn widget_type(&self) -> WidgetType {
        use WidgetType::*;
        match self {
            Widget::List(list) => Into::<WidgetType>::into(list.data_type),
            Widget::Category(categories) => Into::<WidgetType>::into(categories.category),
            Widget::Preview(_) => Preview,
        }
    }
}
