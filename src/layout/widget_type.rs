use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    Input,
    List,
    Done,
    Project,
    Context,
}

