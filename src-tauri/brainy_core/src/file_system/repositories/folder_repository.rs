use async_trait::async_trait;

use crate::{
    Guid,
    common::repository_error::RepositoryError,
    file_system::{
        entities::folder::Folder, value_objects::file_system_item_name::FileSystemItemName,
    },
};

#[async_trait]
pub trait FolderRepository: Send + Sync {
    async fn get_all_folders(&self) -> Result<Vec<Folder>, RepositoryError>;
    async fn exists(
        &self,
        parent_id: Option<Guid>,
        name: &FileSystemItemName,
    ) -> Result<bool, RepositoryError>;
    async fn create(&self, folder: &Folder) -> Result<(), RepositoryError>;
}
