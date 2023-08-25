use super::state_preview::StatePreview;
use super::{state_categories::StateCategories, state_list::StateList, widget_type::WidgetType};
use crate::layout::widget::widget_base::WidgetBase;
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
}
