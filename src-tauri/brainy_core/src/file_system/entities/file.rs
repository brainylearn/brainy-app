use chrono::{DateTime, Utc};

use crate::{
    Guid,
    file_system::value_objects::{
        file_system_item_name::FileSystemItemName, item_fsrs_profile::ItemFsrsProfile,
    },
};

#[derive(Debug, Clone)]
pub struct File {
    id: Guid,
    created_date: DateTime<Utc>,
    modified_date: DateTime<Utc>,
    parent_id: Option<Guid>,
    name: FileSystemItemName,
    fsrs_profile: ItemFsrsProfile,
}

impl File {
    pub(in crate::file_system) fn new(
        id: Option<Guid>,
        parent_id: Option<Guid>,
        name: FileSystemItemName,
        fsrs_profile: ItemFsrsProfile,
    ) -> File {
        File {
            id: id.unwrap_or(Guid::new_v4()),
            created_date: Utc::now(),
            modified_date: Utc::now(),
            parent_id,
            name,
            fsrs_profile,
        }
    }

    /// Used for unit testing, or repositories when reconstructing a file.
    pub fn new_unchecked(
        id: Guid,
        created_date: DateTime<Utc>,
        modified_date: DateTime<Utc>,
        parent_id: Option<Guid>,
        name: FileSystemItemName,
        fsrs_profile: ItemFsrsProfile,
    ) -> File {
        File {
            id,
            created_date,
            modified_date,
            parent_id,
            name,
            fsrs_profile,
        }
    }

    pub fn id(&self) -> Guid {
        self.id
    }

    pub fn created_date(&self) -> DateTime<Utc> {
        self.created_date
    }

    pub fn modified_date(&self) -> DateTime<Utc> {
        self.modified_date
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

    pub fn fsrs_profile(&self) -> &ItemFsrsProfile {
        &self.fsrs_profile
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn new_with_id_created_file_correctly() {
        // Arrange

        let id = Guid::new_v4();

        // Act

        let actual = File::new(
            Some(id),
            None,
            "test".try_into().unwrap(),
            ItemFsrsProfile::Inherit,
        );

        // Assert

        assert_eq!(id, actual.id());
        assert_eq!(None, actual.parent_id());
        assert_eq!(
            FileSystemItemName::new_unchecked("test".to_string()),
            actual.name()
        );
    }

    #[test]
    fn new_without_id_generated_id_automatically() {
        // Act

        let actual = File::new(
            None,
            Some(Guid::new_v4()),
            FileSystemItemName::new_unchecked("test".to_string()),
            ItemFsrsProfile::Inherit,
        );

        // Assert

        assert_ne!(Guid::nil(), actual.id());
        assert_ne!(None, actual.parent_id());
    }
}
