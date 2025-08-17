use async_trait::async_trait;

use crate::{
    Guid,
    common::repository_error::RepositoryError,
    file_system::{entities::file::File, value_objects::file_system_item_name::FileSystemItemName},
};

// TODO: change name
#[async_trait]
pub trait FileRepository: Send + Sync {
    async fn get_all_files(&self) -> Result<Vec<File>, RepositoryError>;
    async fn exists(
        &self,
        parent_id: Option<Guid>,
        name: &FileSystemItemName,
    ) -> Result<bool, RepositoryError>;
    async fn create(&self, file: &File) -> Result<(), RepositoryError>;
}
