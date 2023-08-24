use super::state_preview::StatePreview;
use super::{state_categories::StateCategories, state_list::StateList, widget_type::WidgetType};
use crate::todo::{ToDo, ToDoCategory, ToDoData};
use crate::CONFIG;
use std::sync::{Arc, Mutex};

pub type RCToDo = Arc<Mutex<ToDo>>;

#[enum_dispatch(State)]
pub enum WidgetState {
    List(StateList),
    Category(StateCategories),
    Preview(StatePreview),
}

impl WidgetState {
    pub fn new(widget_type: &WidgetType, data: RCToDo) -> Self {
        match widget_type {
            WidgetType::List => Self::List(StateList::new(
                ToDoData::Pending,
                data,
                CONFIG
                    .list_active_color
                    .combine(&CONFIG.pending_active_color)
                    .get_style(),
                CONFIG.list_shift,
                CONFIG.pending_sort,
            )),
            WidgetType::Done => Self::List(StateList::new(
                ToDoData::Done,
                data,
                CONFIG
                    .list_active_color
                    .combine(&CONFIG.done_active_color)
                    .get_style(),
                CONFIG.list_shift,
                CONFIG.done_sort,
            )),
            WidgetType::Project => {
                Self::Category(StateCategories::new(ToDoCategory::Projects, data))
            }
            WidgetType::Context => {
                Self::Category(StateCategories::new(ToDoCategory::Contexts, data))
            }
            WidgetType::Hashtag => {
                Self::Category(StateCategories::new(ToDoCategory::Hashtags, data))
            }
            WidgetType::Preview => Self::Preview(StatePreview::new(
                "Pending: {n}   Done: {N}\nSubject: {s}\nPriority: {p}\nCreate date: {c}",
                data,
            )),
        }
    }
}
