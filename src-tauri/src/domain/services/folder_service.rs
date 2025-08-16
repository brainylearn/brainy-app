use brainy_core::{common::repository_error::RepositoryError, file_system::{folder_repository::FolderRepository, path::Path}};
use thiserror::Error;


pub struct FolderService<FR: FolderRepository> {
    folder_repository: FR,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    RepositoryError(#[from] RepositoryError),
    #[error("Folder already exists")]
    FolderAlreadyExists,
}

impl<FR: FolderRepository> FolderService<FR> {
    pub fn new(folder_repository: FR) -> Self {
        Self { folder_repository }
    }

    pub async fn create_folder(&self, path: Path) -> Result<(), Error> {
        if self.folder_repository.folder_exists(&path).await? {
            return Err(Error::FolderAlreadyExists);
        }
        // TODO: transactions

        Ok(())
    }
}
