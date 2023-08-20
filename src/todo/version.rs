use crate::todo::ToDoData;

#[derive(Default, PartialEq)]
pub struct Version {
    pending: usize,
    done: usize,
}

pub enum VersionUpdate {
    None, Data(ToDoData), Both
}

impl Version {
    pub fn update(&mut self, other: &Self) -> VersionUpdate {
        let mut ret = VersionUpdate::None;
        if self.pending < other.pending {
            self.pending = other.pending;
            ret = VersionUpdate::Data(ToDoData::Pending);
        }
        if self.done < other.done {
            self.done = other.done;
            if let VersionUpdate::Data(_) = ret {
                ret = VersionUpdate::Both;
            } else {
                ret = VersionUpdate::Data(ToDoData::Pending);
            }
        }
        ret
    }
}

