mod state_categories;
mod state_list;
mod state_preview;
mod widget_base;
mod widget_list;
mod widget_trait;
mod widget_type;

use crate::{
    config::Config,
    layout::widget::widget_list::WidgetList,
    todo::{ToDoCategory, ToDoData},
    Result,
};
use state_categories::StateCategories;
use state_list::StateList;
use state_preview::StatePreview;
use widget_base::WidgetBase;
pub use widget_trait::State;
pub use widget_type::WidgetType;

pub fn new_widget(widget_type: WidgetType, config: &Config) -> Result<Box<dyn State>> {
    use WidgetType::*;
    Ok(match widget_type {
        List => Box::new(StateList::new(
            WidgetList::new(&widget_type, config),
            ToDoData::Pending,
            config,
        )?),
        Done => Box::new(StateList::new(
            WidgetList::new(&widget_type, config),
            ToDoData::Done,
            config,
        )?),
        Project => Box::new(StateCategories::new(
            WidgetList::new(&widget_type, config),
            ToDoCategory::Projects,
            &config.active_color_config,
        )),
        Context => Box::new(StateCategories::new(
            WidgetList::new(&widget_type, config),
            ToDoCategory::Contexts,
            &config.active_color_config,
        )),
        Hashtag => Box::new(StateCategories::new(
            WidgetList::new(&widget_type, config),
            ToDoCategory::Hashtags,
            &config.active_color_config,
        )),
        Preview => Box::new(StatePreview::new(
            WidgetBase::new(&widget_type, config),
            config,
        )?),
    })
}
