use async_trait::async_trait;

use crate::{common::repository_error::RepositoryError, file_system::{folder::Folder, path::Path}};

#[async_trait]
pub trait FolderRepository: Send + Sync {
    async fn get_all_files(&self) -> Result<Vec<Folder>, RepositoryError>;
    async fn get_by_path(&self, path: &Path) -> Result<Option<Folder>, RepositoryError>;
    async fn folder_exists(&self, path: &Path) -> Result<bool, RepositoryError>;
}
