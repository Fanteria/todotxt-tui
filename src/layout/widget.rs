mod state_categories;
mod state_list;
mod state_preview;
mod widget_base;
mod widget_list;
mod widget_trait;
mod widget_type;

use crate::{
    config::Config,
    layout::widget::{
        state_preview::{ActivePreview, DoneActualPreview, PendingActualPreview},
        widget_list::WidgetList,
    },
    todo::{ToDoCategory, ToDoData},
};
use anyhow::Result;
use state_categories::StateCategories;
use state_list::StateList;
use state_preview::StatePreview;
use widget_base::WidgetBase;
pub use widget_trait::State;
pub use widget_type::WidgetType;

pub fn new_widget(widget_type: WidgetType, config: &Config) -> Result<Box<dyn State>> {
    let wb_config = &config.widget_base_config;
    use WidgetType::*;
    Ok(match widget_type {
        List => Box::new(StateList::new(
            WidgetList::new(
                WidgetBase::new(&wb_config.pending_widget_name, config)
                    .events(wb_config.tasks_keybind.clone()),
                config,
            ),
            ToDoData::Pending,
            config,
        )?),
        Done => Box::new(StateList::new(
            WidgetList::new(
                WidgetBase::new(&wb_config.done_widget_name, config)
                    .events(wb_config.tasks_keybind.clone()),
                config,
            ),
            ToDoData::Done,
            config,
        )?),
        Project => Box::new(StateCategories::new(
            WidgetList::new(
                WidgetBase::new(&wb_config.project_widget_name, config)
                    .events(wb_config.category_keybind.clone()),
                config,
            ),
            ToDoCategory::Projects,
            &config.active_color_config,
        )),
        Context => Box::new(StateCategories::new(
            WidgetList::new(
                WidgetBase::new(&wb_config.context_widget_name, config)
                    .events(wb_config.category_keybind.clone()),
                config,
            ),
            ToDoCategory::Contexts,
            &config.active_color_config,
        )),
        Hashtag => Box::new(StateCategories::new(
            WidgetList::new(
                WidgetBase::new(&wb_config.hashtag_widget_name, config)
                    .events(wb_config.category_keybind.clone()),
                config,
            ),
            ToDoCategory::Hashtags,
            &config.active_color_config,
        )),
        Preview => Box::new(StatePreview::<ActivePreview>::new(
            WidgetBase::new(&config.widget_base_config.preview_widget_name, config),
            config,
        )?),
        PendingLivePreview => Box::new(StatePreview::<PendingActualPreview>::new(
            WidgetBase::new(
                &config.widget_base_config.pending_live_preview_widget_name,
                config,
            ),
            config,
        )?),
        DoneLivePreview => Box::new(StatePreview::<DoneActualPreview>::new(
            WidgetBase::new(
                &config.widget_base_config.done_live_preview_widget_name,
                config,
            ),
            config,
        )?),
    })
}
