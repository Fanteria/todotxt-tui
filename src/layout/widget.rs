mod state_categories;
mod state_list;
mod state_preview;
mod widget_base;
mod widget_list;
pub mod widget_trait;
pub mod widget_type;

use crate::{
    layout::widget::widget_list::WidgetList,
    todo::{ToDo, ToDoCategory, ToDoData},
    ui::UIEvent,
    CONFIG,
};
use crossterm::event::KeyCode;
use state_categories::StateCategories;
use state_list::StateList;
use state_preview::StatePreview;
use std::sync::{Arc, Mutex, MutexGuard};
use tui::widgets::Block;
use tui::{backend::Backend, Frame};
use widget_base::WidgetBase;
pub use widget_trait::State;
use widget_type::WidgetType;

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
        match widget_type {
            List => Self::List(StateList::new(
                WidgetList::new(&widget_type, data),
                ToDoData::Pending,
                CONFIG
                    .list_active_color
                    .combine(&CONFIG.pending_active_color)
                    .get_style(),
                CONFIG.list_shift,
                CONFIG.pending_sort,
            )),
            Done => Self::List(StateList::new(
                WidgetList::new(&widget_type, data),
                ToDoData::Done,
                CONFIG
                    .list_active_color
                    .combine(&CONFIG.done_active_color)
                    .get_style(),
                CONFIG.list_shift,
                CONFIG.done_sort,
            )),
            Project => Self::Category(StateCategories::new(
                WidgetList::new(&widget_type, data),
                ToDoCategory::Projects,
            )),
            Context => Self::Category(StateCategories::new(
                WidgetList::new(&widget_type, data),
                ToDoCategory::Contexts,
            )),
            Hashtag => Self::Category(StateCategories::new(
                WidgetList::new(&widget_type, data),
                ToDoCategory::Hashtags,
            )),
            Preview => Self::Preview(StatePreview::new(
                WidgetBase::new(&widget_type, data),
                CONFIG.preview_format.clone(),
            )),
        }
    }

    pub fn widget_type(&self) -> WidgetType {
        use WidgetType::*;
        match self {
            Widget::List(list) => list.data_type.into(),
            Widget::Category(categories) => categories.category.into(),
            Widget::Preview(_) => Preview,
        }
    }
}
