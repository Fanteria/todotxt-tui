use crate::{
    layout::widget::widget_type::WidgetType,
    layout::Layout,
    todo::{ToDo, ToDoState},
    {Result, ToDoIoError},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Result as ioResult, Write},
    path::Path,
};

#[derive(Default, Serialize, Deserialize)]
pub struct UIState {
    pub active: WidgetType,
    pub todo_state: ToDoState,
}

impl UIState {
    pub fn new(layout: &Layout, todo: &ToDo) -> Self {
        Self {
            active: layout.get_active_widget(),
            todo_state: todo.get_state().clone(),
        }
    }

    pub fn save(&self, path: &Path) -> ioResult<()> {
        self.serialize(&mut File::create(path)?)
    }

    fn serialize<W: Write>(&self, writer: &mut W) -> ioResult<()> {
        writer.write_all(toml::to_string_pretty(&self).unwrap().as_bytes())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|err| ToDoIoError {
            path: path.to_path_buf(),
            err,
        })?;
        Ok(UIState::deserialize(file))
    }

    fn deserialize<R: Read>(mut reader: R) -> Self {
        let mut buf = String::default();
        if let Err(e) = reader.read_to_string(&mut buf) {
            log::error!("Cannot load UI state: {}", e);
            return Self::default();
        }
        match toml::from_str(buf.as_str()) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Cannot parse UI state: {}", e);
                Self::default()
            }
        }
    }
}
