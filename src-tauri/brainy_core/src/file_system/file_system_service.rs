use async_trait::async_trait;
use thiserror::Error;

use crate::{
    Guid,
    common::repository_error::RepositoryError,
    file_system::{
        entities::{file::File, folder::Folder},
        repositories::{file_repository::FileRepository, folder_repository::FolderRepository},
        value_objects::file_system_item_name::FileSystemItemName,
    },
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("The file with the name '{name}' could not be found!")]
    FileNotFound { name: String },
    #[error("The file with the name '{name}' already exists!")]
    FileExists { name: String },

    #[error("The folder with the name '{name}' could not be found!")]
    FolderNotFound { name: String },
    #[error("The folder with the name '{name}' already exists!")]
    FolderExists { name: String },

    #[error("{0}")]
    UnknownRepositoryError(#[from] RepositoryError),
}

#[async_trait]
pub trait FileSystemService: Send + Sync {
    async fn create_folder(
        &self,
        parent_id: Option<Guid>,
        name: FileSystemItemName,
    ) -> Result<Guid, Error>;

    async fn create_file(
        &self,
        parent_id: Option<Guid>,
        name: FileSystemItemName,
    ) -> Result<Guid, Error>;
}

pub struct DefaultFileSystemService {
    // TODO: not pub
    pub folder_repository: Box<dyn FolderRepository>,
    pub file_repository: Box<dyn FileRepository>,
}

#[async_trait]
impl FileSystemService for DefaultFileSystemService {
    async fn create_folder(
        &self,
        parent_id: Option<Guid>,
        name: FileSystemItemName,
    ) -> Result<Guid, Error> {
        if self.folder_repository.exists(parent_id, &name).await? {
            return Err(Error::FolderExists {
                name: name.to_string(),
            });
        }

        let folder = Folder::new(None, parent_id, name);
        self.folder_repository.create(&folder).await?;

        Ok(folder.id())
    }

    async fn create_file(
        &self,
        parent_id: Option<Guid>,
        name: FileSystemItemName,
    ) -> Result<Guid, Error> {
        if self.file_repository.exists(parent_id, &name).await? {
            return Err(Error::FileExists {
                name: name.to_string(),
            });
        }

        let file = File::new(None, parent_id, name);
        self.file_repository.create(&file).await?;

        Ok(file.id())
    }
}
