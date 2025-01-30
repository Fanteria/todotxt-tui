use crate::{
    layout::{widget::widget_type::WidgetType, Layout},
    todo::{ToDo, ToDoState},
    Result, ToDoError,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq)]
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
    pub fn save(&self, path: &Path) -> Result<()> {
        self.serialize(
            &mut File::create(path).map_err(|err| ToDoError::io_operation_failed(path, err))?,
        )
    }

    /// Serializes the current `UIState` to a writer using TOML format for easy
    /// human-readable serialization and deserialization.
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(toml::to_string_pretty(&self).unwrap().as_bytes())?;
        Ok(())
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
            log::error!("Cannot load UI state, invalid UTF-8: {}", e);
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

#[cfg(test)]
mod tests {
    use crate::config::Config;

    use super::*;
    use std::{io::BufWriter, path::PathBuf};
    use test_log::test;

    #[test]
    fn non_existing_state_file() {
        if let ToDoError::IOoperationFailed(path, _) =
            UIState::load(&PathBuf::from("/this/path/does/not/exists"))
                .expect_err("This file should not exists.")
        {
            assert_eq!(path, PathBuf::from("/this/path/does/not/exists"))
        } else {
            panic!("Load returns unexpected error");
        }

        if let ToDoError::IOoperationFailed(path, _) = UIState::default()
            .save(&PathBuf::from("/this/path/does/not/exists"))
            .expect_err("This file should not exists.")
        {
            assert_eq!(path, PathBuf::from("/this/path/does/not/exists"))
        } else {
            panic!("Load returns unexpected error");
        }
    }

    #[test]
    fn de_serialize() -> Result<()> {
        let layout = Layout::from_str("[ Done, ]", &ToDo::default(), &Config::default()).unwrap();
        let state = UIState::new(&layout, &ToDo::default());

        let mut writer = BufWriter::new(Vec::new());
        state.serialize(&mut writer)?;
        let deserialized = UIState::deserialize(writer.into_inner().unwrap().as_slice());
        assert_ne!(deserialized, UIState::default());
        assert_eq!(deserialized.active, WidgetType::Done);
        Ok(())
    }

    #[test]
    fn deserialize_failed() {
        let s = String::from("invalid toml");
        assert_eq!(UIState::deserialize(s.as_bytes()), UIState::default());

        let invalid_utf8: Vec<u8> = vec![0xf0, 0x28, 0x8c, 0x28];
        assert_eq!(
            UIState::deserialize(invalid_utf8.as_slice()),
            UIState::default()
        );
    }
}
