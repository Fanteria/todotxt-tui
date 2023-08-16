use super::state_preview::StatePreview;
use super::{
    state_categories::StateCategories, state_input::StateInput, state_list::StateList,
    widget_type::WidgetType,
};
use crate::todo::{ToDo, ToDoCategory, ToDoData};
use std::sync::{Arc, Mutex};

pub type RCToDo = Arc<Mutex<ToDo>>;

#[enum_dispatch(State)]
pub enum WidgetState {
    Input(StateInput),
    List(StateList),
    Category(StateCategories),
    Preview(StatePreview),
}

impl WidgetState {
    pub fn new(widget_type: &WidgetType, data: RCToDo) -> Self {
        match widget_type {
            WidgetType::Input => Self::Input(StateInput::new(data)),
            WidgetType::List => Self::List(StateList::new(ToDoData::Pending, data)),
            WidgetType::Done => Self::List(StateList::new(ToDoData::Done, data)),
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
