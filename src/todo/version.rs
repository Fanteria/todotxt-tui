use std::sync::mpsc::Sender;

use crate::file_worker::FileWorkerCommands;

use super::ToDoData;

type VersionNum = usize;

#[derive(Default, Debug, Clone, Copy)]
pub struct Versions {
    pending: VersionNum,
    done: VersionNum,
}

#[derive(Default, Debug)]
pub struct Version {
    versions: Versions,
    pub tx: Option<Sender<FileWorkerCommands>>,
}

impl Version {
    pub fn new(tx: Sender<FileWorkerCommands>) -> Self {
        Self {
            versions: Versions::default(),
            tx: Some(tx),
        }
    }

    pub fn update_all(&mut self) {
        self.versions.pending += 1;
        self.versions.done += 1;
        // TODO save file
    }

    pub fn update(&mut self, data_type: &ToDoData) {
        match data_type {
            ToDoData::Pending => {
                self.versions.pending += 1;
            }
            ToDoData::Done => self.versions.done += 1,
        }
    }

    pub fn is_actual(&self, version: VersionNum, data_type: &ToDoData) -> bool {
        version
            == match data_type {
                ToDoData::Pending => self.versions.pending,
                ToDoData::Done => self.versions.done,
            }
    }

    pub fn is_actual_all(&self, versions: Versions) -> bool {
        self.versions.pending == versions.pending && self.versions.done == versions.done
    }

    pub fn get_version(&self, data_type: &ToDoData) -> VersionNum {
        match data_type {
            ToDoData::Pending => self.versions.pending,
            ToDoData::Done => self.versions.done,
        }
    }

    pub fn get_version_all(&self) -> Versions {
        self.versions
    }
}
