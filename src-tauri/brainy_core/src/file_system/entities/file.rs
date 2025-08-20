use crate::{Guid, file_system::value_objects::file_system_item_name::FileSystemItemName};

// TODO: on the same aggregate for file and not folder, add cells and reviews and parent_id
#[derive(Debug, Clone)]
pub struct File {
    id: Guid,
    parent_id: Option<Guid>,
    name: FileSystemItemName,
}

impl File {
    pub(in crate::file_system) fn new(
        id: Option<Guid>,
        parent_id: Option<Guid>,
        name: FileSystemItemName,
    ) -> File {
        File {
            id: id.unwrap_or(Guid::new_v4().into()),
            parent_id: parent_id,
            name,
        }
    }

    /// A method to create a folder, this should only be used be repositories when
    /// reconstructing a folder. Otherwise use `FileSystemService` for creating files.
    pub fn new_unchecked(
        id: Option<Guid>,
        parent_id: Option<Guid>,
        name: FileSystemItemName,
    ) -> File {
        File {
            id: id.unwrap_or(Guid::new_v4().into()),
            parent_id: parent_id,
            name,
        }
    }

    pub fn id(&self) -> Guid {
        self.id
    }

    pub fn parent_id(&self) -> Option<Guid> {
        self.parent_id
    }

    pub fn name(&self) -> FileSystemItemName {
        self.name.clone()
    }

    pub(in crate::file_system) fn set_name(&mut self, new_name: FileSystemItemName) {
        self.name = new_name;
    }

    pub(in crate::file_system) fn set_parent_id(&mut self, parent_id: Option<Guid>) {
        self.parent_id = parent_id;
    }
}
