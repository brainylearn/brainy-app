use async_trait::async_trait;

use crate::domain::{
    entities::folder::Folder, repositories::repository_error::RepositoryError,
    value_objects::path::Path,
};

#[async_trait]
pub trait FolderRepository: Send + Sync {
    async fn upsert(&mut self, folder: &Folder) -> Result<(), RepositoryError>;
    async fn get_by_path(&self, path: &Path) -> Result<Option<Folder>, RepositoryError>;
    async fn folder_exists(&self, path: &Path) -> Result<bool, RepositoryError>;
}
