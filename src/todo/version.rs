use super::ToDoData;
use crate::file_worker::FileWorkerCommands;
use std::sync::mpsc::Sender;

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
    /// Constructs a new `Version` instance.
    pub fn new(tx: Sender<FileWorkerCommands>) -> Self {
        Self {
            versions: Versions::default(),
            tx: Some(tx),
        }
    }

    /// Update version of both pending and done tasks.
    pub fn update_all(&mut self) {
        self.versions.pending += 1;
        self.versions.done += 1;
        if let Some(tx) = &self.tx {
            if let Err(e) = tx.send(FileWorkerCommands::Save) {
                log::error!("Error while send signal to save todo list from update all: {e}");
            }
        }
    }

    /// Update version of a specific task type.
    pub fn update(&mut self, data_type: &ToDoData) {
        match data_type {
            ToDoData::Pending => {
                self.versions.pending += 1;
            }
            ToDoData::Done => self.versions.done += 1,
        }
        if let Some(tx) = &self.tx {
            if let Err(e) = tx.send(FileWorkerCommands::Save) {
                log::error!("Error while send signal to save todo list from update: {e}");
            }
        }
    }

    /// Check if the version is actual.
    pub fn is_actual(&self, version: VersionNum, data_type: &ToDoData) -> bool {
        version
            == match data_type {
                ToDoData::Pending => self.versions.pending,
                ToDoData::Done => self.versions.done,
            }
    }

    /// Check if the version is actual for both pending and done tasks.
    pub fn is_actual_all(&self, versions: Versions) -> bool {
        self.versions.pending == versions.pending && self.versions.done == versions.done
    }

    /// Get the version of a specific task type.
    pub fn get_version(&self, data_type: &ToDoData) -> VersionNum {
        match data_type {
            ToDoData::Pending => self.versions.pending,
            ToDoData::Done => self.versions.done,
        }
    }

    /// Get the version of both pending and done tasks.
    pub fn get_version_all(&self) -> Versions {
        self.versions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn behaviour() {
        let mut v = Version {
            versions: Versions::default(),
            tx: None,
        };

        v.update(&ToDoData::Pending);
        assert_eq!(v.get_version(&ToDoData::Pending), 1);
        assert_eq!(v.get_version(&ToDoData::Done), 0);

        v.update_all();
        assert_eq!(v.get_version(&ToDoData::Pending), 2);
        assert_eq!(v.get_version(&ToDoData::Done), 1);

        v.update_all();
        assert_eq!(v.get_version(&ToDoData::Pending), 3);
        assert_eq!(v.get_version(&ToDoData::Done), 2);

        let mut new_v = Version {
            versions: v.get_version_all(),
            tx: None,
        };
        v.update_all();
        new_v.update_all();
        v.is_actual_all(new_v.get_version_all());
    }
}
