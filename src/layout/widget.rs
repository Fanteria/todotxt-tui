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
    config::Config,
};
use crossterm::event::KeyCode;
use state_categories::StateCategories;
use state_list::StateList;
use state_preview::StatePreview;
use std::sync::{Arc, Mutex};
use tui::widgets::Block;
use tui::{backend::Backend, Frame};
use widget_base::WidgetBase;
pub use widget_trait::State;
use widget_type::WidgetType;

/// Alias for the shared mutable reference to a ToDo instance.
pub type RCToDo = Arc<Mutex<ToDo>>;

/// Implement the enum_dispatch macro for the State trait.
#[enum_dispatch(State)]
pub enum Widget {
    List(StateList),
    Category(StateCategories),
    Preview(StatePreview),
}

impl Widget {
    /// Create a new widget based on its type and shared ToDo data.
    ///
    /// This function creates a new widget based on its type and the shared ToDo data.
    ///
    /// # Parameters
    ///
    /// - `widget_type`: The type of widget to create.
    /// - `data`: A shared mutable reference to the ToDo data.
    ///
    /// # Returns
    ///
    /// Returns a new instance of the specified widget type.
    pub fn new(widget_type: WidgetType, data: RCToDo, config: &Config) -> Self {
        use WidgetType::*;
        match widget_type {
            List => Self::List(StateList::new(
                WidgetList::new(&widget_type, data, config),
                ToDoData::Pending,
                config
                    .list_active_color
                    .combine(&config.pending_active_color)
                    .get_style(),
                config.list_shift,
                config.pending_sort,
            )),
            Done => Self::List(StateList::new(
                WidgetList::new(&widget_type, data, &config),
                ToDoData::Done,
                config
                    .list_active_color
                    .combine(&config.done_active_color)
                    .get_style(),
                config.list_shift,
                config.done_sort,
            )),
            Project => Self::Category(StateCategories::new(
                WidgetList::new(&widget_type, data, config),
                ToDoCategory::Projects,
            )),
            Context => Self::Category(StateCategories::new(
                WidgetList::new(&widget_type, data, config),
                ToDoCategory::Contexts,
            )),
            Hashtag => Self::Category(StateCategories::new(
                WidgetList::new(&widget_type, data, config),
                ToDoCategory::Hashtags,
            )),
            Preview => Self::Preview(StatePreview::new(
                WidgetBase::new(&widget_type, data, config),
                config.preview_format.clone(),
                config.wrap_preview,
            ).unwrap()), // TODO produce error
        }
    }

    /// Get the type of the widget.
    ///
    /// This function returns the type of the widget.
    ///
    /// # Returns
    ///
    /// Returns the `WidgetType` of the widget.
    pub fn widget_type(&self) -> WidgetType {
        use WidgetType::*;
        match self {
            Widget::List(list) => list.data_type.into(),
            Widget::Category(categories) => categories.category.into(),
            Widget::Preview(_) => Preview,
        }
    }
}
