use crate::{Guid, file_system::value_objects::file_system_item_name::FileSystemItemName};

#[derive(Debug, Clone)]
pub struct Folder {
    id: Guid,
    // TODO: Maybe use an enum?
    /// If the parent is None, then the parent is the root.
    parent_id: Option<Guid>,
    name: FileSystemItemName,
}

impl Folder {
    pub(in crate::file_system) fn new(
        id: Option<Guid>,
        parent_id: Option<Guid>,
        name: FileSystemItemName,
    ) -> Folder {
        Folder {
            id: id.unwrap_or(Guid::new_v4().into()),
            parent_id: parent_id,
            name,
        }
    }

    /// A method to create a folder, this should only be used be repositories when
    /// reconstructing a folder. Otherwise use `FileSystemService` for creating folders.
    pub fn new_unchecked(
        id: Option<Guid>,
        parent_id: Option<Guid>,
        name: FileSystemItemName,
    ) -> Folder {
        Folder {
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
}
