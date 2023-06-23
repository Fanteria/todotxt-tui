use super::{
    state_categories::StateCategories, state_input::StateInput, state_list::StateList,
    widget_type::WidgetType,
};
use crate::todo::{CategoryList, ToDo};
use std::cell::RefCell;
use std::rc::Rc;

pub type RCToDo = Rc<RefCell<ToDo>>;

#[enum_dispatch(State)]
pub enum WidgetState {
    Input(StateInput),
    List(StateList),
    Category(StateCategories),
}

impl WidgetState {
    pub fn new_category(
        fn_list: fn(&ToDo) -> CategoryList,
        fn_toggle: fn(&mut ToDo, &str),
        data: RCToDo,
    ) -> Self {
        Self::Category(StateCategories::new(fn_list, fn_toggle, data))
    }

    pub fn new(widget_type: &WidgetType, data: RCToDo) -> Self {
        match widget_type {
            WidgetType::Input => WidgetState::Input(StateInput::new(data)),
            WidgetType::List => WidgetState::List(StateList::new(
                |todo| todo.get_pending_filtered(),
                |todo, i| todo.move_pending_task(i),
                data,
            )),
            WidgetType::Done => WidgetState::List(StateList::new(
                |todo| todo.get_done_filtered(),
                |todo, i| todo.move_done_task(i),
                data,
            )),
            WidgetType::Project => WidgetState::new_category(
                |todo| todo.get_projects(),
                |todo, category| ToDo::toggle_filter(&mut todo.project_filters, category),
                data,
            ),
            WidgetType::Context => WidgetState::new_category(
                |todo| todo.get_contexts(),
                |todo, category| ToDo::toggle_filter(&mut todo.context_filters, category),
                data,
            ),
            WidgetType::Hashtag => WidgetState::new_category(
                |todo| todo.get_hashtags(),
                |todo, category| ToDo::toggle_filter(&mut todo.hashtag_filters, category),
                data,
            ),
        }
    }
}
