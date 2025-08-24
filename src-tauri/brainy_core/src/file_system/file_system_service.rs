use std::sync::Arc;

// TODO: unit test
use thiserror::Error;

use crate::{
    Guid,
    common::repository_error::RepositoryError,
    file_system::{
        entities::{file::File, folder::Folder},
        repositories::traits::{
            file_repository::FileRepository, folder_repository::FolderRepository,
        },
        value_objects::file_system_item_name::FileSystemItemName,
    },
};

#[derive(Error, Debug)]
pub enum FileServiceError {
    #[error("The file with the name '{name}' already exists!")]
    FileExists { name: String },
    #[error("The folder with the name '{name}' already exists!")]
    FolderExists { name: String },
    #[error("{0}")]
    UnknownRepositoryError(#[from] RepositoryError),
}

pub struct FileSystemService {
    folder_repository: Arc<dyn FolderRepository>,
    file_repository: Arc<dyn FileRepository>,
}

impl FileSystemService {
    pub fn new(
        folder_repository: Arc<dyn FolderRepository>,
        file_repository: Arc<dyn FileRepository>,
    ) -> Self {
        Self {
            folder_repository,
            file_repository,
        }
    }

    pub async fn create_folder(
        &self,
        parent_id: Option<Guid>,
        name: FileSystemItemName,
    ) -> Result<Guid, FileServiceError> {
        log::info!(
            "Creating folder with name {name} and inside parent folder {:?}",
            parent_id
        );

        if self.folder_repository.exists(parent_id, &name).await? {
            return Err(FileServiceError::FolderExists {
                name: name.to_string(),
            });
        }

        let folder = Folder::new(None, parent_id, name);
        self.folder_repository.create(&folder).await?;

        log::info!("Created folder with id {}", folder.id());
        Ok(folder.id())
    }

    pub async fn rename_folder(
        &self,
        folder_id: Guid,
        new_name: FileSystemItemName,
    ) -> Result<(), FileServiceError> {
        log::info!("Renaming folder with id {folder_id} into name {new_name}");

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
            return Err(FileServiceError::FolderExists {
                name: new_name.to_string(),
            });
        }

        folder.set_name(new_name.clone());
        self.folder_repository.update(&folder).await?;
        log::info!("Renamed folder with id {folder_id} to {new_name}");
        Ok(())
    }

    pub async fn move_folder(
        &self,
        folder_id: Guid,
        destination_folder_id: Option<Guid>,
    ) -> Result<(), FileServiceError> {
        log::info!(
            "Moving folder with id {folder_id} into folder with id {:?}",
            destination_folder_id
        );

        let mut folder = self.folder_repository.get_by_id(folder_id).await?;

        if Some(folder_id) == destination_folder_id || folder.parent_id() == destination_folder_id {
            log::info!("Skip moving the folder into the same folder!");
            return Ok(());
        }

        if self
            .folder_repository
            .exists(destination_folder_id, &folder.name())
            .await?
        {
            return Err(FileServiceError::FolderExists {
                name: folder.name().to_string(),
            });
        }

        // TODO: stop a folder from being moved into a folder inside it
        folder.set_parent_id(destination_folder_id);
        self.folder_repository.update(&folder).await?;
        log::info!(
            "Moved folder with name {}, and id {:?} from folder with id {:?} to folder with id {:?}",
            folder.name(),
            folder_id,
            folder.parent_id(),
            destination_folder_id
        );
        Ok(())
    }

    pub async fn create_file(
        &self,
        parent_id: Option<Guid>,
        name: FileSystemItemName,
    ) -> Result<Guid, FileServiceError> {
        log::info!(
            "Creating file with name {name} and inside parent folder {:?}",
            parent_id
        );

        if self.file_repository.exists(parent_id, &name).await? {
            return Err(FileServiceError::FileExists {
                name: name.to_string(),
            });
        }

        let file = File::new(None, parent_id, name);
        self.file_repository.create(&file).await?;
        log::info!("Created file with id {}", file.id());

        Ok(file.id())
    }

    pub async fn rename_file(
        &self,
        file_id: Guid,
        new_name: FileSystemItemName,
    ) -> Result<(), FileServiceError> {
        log::info!("Renaming file with id {file_id} into name {new_name}");

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
            return Err(FileServiceError::FileExists {
                name: new_name.to_string(),
            });
        }

        file.set_name(new_name.clone());
        self.file_repository.update(&file).await?;
        log::info!("Renamed file with id {file_id} to {new_name}");
        Ok(())
    }

    pub async fn move_file(
        &self,
        file_id: Guid,
        destination_folder_id: Option<Guid>,
    ) -> Result<(), FileServiceError> {
        log::info!(
            "Moving file with id {file_id} into folder with id {:?}",
            destination_folder_id
        );

        let mut file = self.file_repository.get_by_id(file_id).await?;

        if file.parent_id() == destination_folder_id {
            log::info!("Skip moving the file into the same folder!");
            return Ok(());
        }

        if self
            .file_repository
            .exists(destination_folder_id, &file.name())
            .await?
        {
            return Err(FileServiceError::FileExists {
                name: file.name().to_string(),
            });
        }

        file.set_parent_id(destination_folder_id);
        self.file_repository.update(&file).await?;
        log::info!(
            "Moved file with name {}, and id {:?} from folder with id {:?} to folder with id {:?}",
            file.name(),
            file_id,
            file.parent_id(),
            destination_folder_id
        );
        Ok(())
    }
}
