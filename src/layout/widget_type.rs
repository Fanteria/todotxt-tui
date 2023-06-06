use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    Input,
    List,
    Done,
    Project,
    Context,
}
