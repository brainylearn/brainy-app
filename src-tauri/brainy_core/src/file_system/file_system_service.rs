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
    #[error("The file with the name '{name}' already exists!")]
    FileExists { name: String },
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

    async fn rename_folder(&self, folder_id: Guid, new_name: FileSystemItemName) -> Result<(), Error>;

    async fn create_file(
        &self,
        parent_id: Option<Guid>,
        name: FileSystemItemName,
    ) -> Result<Guid, Error>;

    async fn rename_file(&self, file_id: Guid, new_name: FileSystemItemName) -> Result<(), Error>;
}

pub struct DefaultFileSystemService {
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

    async fn rename_folder(&self, folder_id: Guid, new_name: FileSystemItemName) -> Result<(), Error> {
        let mut folder = self.folder_repository.get_by_id(folder_id).await?;

        if folder.name() == new_name {
            log::info!("Skip renaming since the name is the same!");
            return Ok(());
        }

        if self
            .folder_repository
            .exists(folder.parent_id(), &new_name)
            .await?
        {
            return Err(Error::FolderExists {
                name: new_name.to_string(),
            });
        }

        folder.set_name(new_name.clone());
        self.folder_repository.update(&folder).await?;
        log::info!("Renamed folder with id {folder_id} to {new_name}");
        Ok(())
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

    async fn rename_file(&self, file_id: Guid, new_name: FileSystemItemName) -> Result<(), Error> {
        let mut file = self.file_repository.get_by_id(file_id).await?;

        if file.name() == new_name {
            log::info!("Skip renaming since the name is the same!");
            return Ok(());
        }

        if self
            .file_repository
            .exists(file.parent_id(), &new_name)
            .await?
        {
            return Err(Error::FileExists {
                name: new_name.to_string(),
            });
        }

        file.set_name(new_name.clone());
        self.file_repository.update(&file).await?;
        log::info!("Renamed file with id {file_id} to {new_name}");
        Ok(())
    }
}
