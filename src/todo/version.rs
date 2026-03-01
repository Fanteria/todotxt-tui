use super::ToDoData;
use crate::file_worker::FileWorkerCommands;
use std::sync::mpsc::Sender;

type VersionNum = usize;

/// Snapshot of version numbers for both pending and done data.
#[derive(Default, Debug, Clone, Copy)]
pub struct Versions {
    pending: VersionNum,
    done: VersionNum,
}

/// Tracks data version numbers for pending and done lists to detect changes.
/// Optionally sends save signals to the file worker when data is modified.
#[derive(Default, Debug)]
pub struct Version {
    versions: Versions,
    pub tx: Option<Sender<FileWorkerCommands>>,
}

impl Version {
    /// Creates a new `Version` with default version numbers and the given save channel.
    pub fn new(tx: Sender<FileWorkerCommands>) -> Self {
        Self {
            versions: Versions::default(),
            tx: Some(tx),
        }
    }

    /// Increments the pending and done version numbers and signals a save.
    pub fn update_all(&mut self) {
        self.versions.pending += 1;
        self.versions.done += 1;
        if let Some(tx) = &self.tx {
            if let Err(e) = tx.send(FileWorkerCommands::Save) {
                log::error!("Error while send signal to save todo list from update all: {e}");
                self.tx = None;
            }
        }
    }

    /// Increments the version number for the given data type and signals a save.
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
                self.tx = None;
            }
        }
    }

    /// Returns `true` if the given version number matches the current version for the data type.
    pub fn is_actual(&self, version: VersionNum, data_type: &ToDoData) -> bool {
        version
            == match data_type {
                ToDoData::Pending => self.versions.pending,
                ToDoData::Done => self.versions.done,
            }
    }

    /// Returns `true` if both pending and done versions match the given snapshot.
    pub fn is_actual_all(&self, versions: Versions) -> bool {
        self.versions.pending == versions.pending && self.versions.done == versions.done
    }

    /// Returns the current version number for the given data type.
    pub fn get_version(&self, data_type: &ToDoData) -> VersionNum {
        match data_type {
            ToDoData::Pending => self.versions.pending,
            ToDoData::Done => self.versions.done,
        }
    }

    /// Returns a snapshot of all current version numbers.
    pub fn get_version_all(&self) -> Versions {
        self.versions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

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
        assert!(v.is_actual_all(new_v.get_version_all()));
    }

    #[test]
    fn sends_save_signal_on_updates() {
        let (tx, rx) = mpsc::channel();
        let mut v = Version::new(tx);

        v.update(&ToDoData::Pending);
        assert!(matches!(rx.try_recv(), Ok(FileWorkerCommands::Save)));

        v.update(&ToDoData::Done);
        assert!(matches!(rx.try_recv(), Ok(FileWorkerCommands::Save)));

        v.update_all();
        assert!(matches!(rx.try_recv(), Ok(FileWorkerCommands::Save)));
    }
}
