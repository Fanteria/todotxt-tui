use crate::{
    layout::{widget::widget_type::WidgetType, Layout},
    todo::{ToDo, ToDoState},
    Result, ToDoError,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Result as ioResult, Write},
    path::Path,
};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct UIState {
    pub active: WidgetType,
    pub todo_state: ToDoState,
}

impl UIState {
    /// Constructs a new `UIState` instance with the specified layout and ToDo state.
    pub fn new(layout: &Layout, todo: &ToDo) -> Self {
        Self {
            active: layout.get_active_widget(),
            todo_state: todo.get_state().clone(),
        }
    }

    /// Saves the current state of the UI to a file at the specified path by serializing
    /// it to TOML format and writing it to a newly created file handle.
    pub fn save(&self, path: &Path) -> ioResult<()> {
        self.serialize(&mut File::create(path)?)
    }

    /// Serializes the current `UIState` to a writer using TOML format for easy
    /// human-readable serialization and deserialization.
    fn serialize<W: Write>(&self, writer: &mut W) -> ioResult<()> {
        writer.write_all(toml::to_string_pretty(&self).unwrap().as_bytes())
    }

    /// Loads a `UIState` from the specified file path by opening the file
    /// and deserializing it from TOML format.
    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|err| ToDoError::io_operation_failed(path, err))?;
        Ok(UIState::deserialize(file))
    }

    /// Deserializes a `UIState` from a reader by parsing it with TOML.
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
