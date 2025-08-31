use std::sync::Arc;

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

#[derive(Error, Debug, PartialEq, Eq)]
pub enum FileServiceError {
    #[error("The file with the name '{name}' already exists!")]
    FileExists { name: String },
    #[error("The folder with the name '{name}' already exists!")]
    FolderExists { name: String },
    #[error("Cannot move folder to a nested folder within the current folder")]
    CannotMoveChildIntoInnerFolder,
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
        log::info!("Creating folder with name {name} and inside parent folder {parent_id:?}");

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
            "Moving folder with id {folder_id} into folder with id {destination_folder_id:?}"
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

        if let Some(destination_folder_id) = destination_folder_id {
            if self
                .is_subfolder_of(folder_id, destination_folder_id)
                .await?
            {
                return Err(FileServiceError::CannotMoveChildIntoInnerFolder);
            }
        }

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

    /// Checks whether the child folder is inside the parent folder.
    async fn is_subfolder_of(
        &self,
        parent_folder_id: Guid,
        child_folder_id: Guid,
    ) -> Result<bool, FileServiceError> {
        let mut curr_parent_id = Some(child_folder_id);

        while curr_parent_id != Some(parent_folder_id) && curr_parent_id.is_some() {
            let curr_folder = self
                .folder_repository
                .get_by_id(curr_parent_id.unwrap())
                .await?;
            curr_parent_id = curr_folder.parent_id();
        }

        Ok(curr_parent_id == Some(parent_folder_id))
    }

    pub async fn create_file(
        &self,
        parent_id: Option<Guid>,
        name: FileSystemItemName,
    ) -> Result<Guid, FileServiceError> {
        log::info!("Creating file with name {name} and inside parent folder {parent_id:?}");

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
        log::info!("Moving file with id {file_id} into folder with id {destination_folder_id:?}");

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

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        ROOT_FOLDER_ID,
        common::{
            sqlite_repositories_context::SqliteRepositoriesContext,
            traits::repositories_context::RepositoriesContext,
        },
    };

    async fn create_test_dependencies() -> (SqliteRepositoriesContext, FileSystemService) {
        let context = SqliteRepositoriesContext::create_testing_context().await;
        let service =
            FileSystemService::new(context.folder_repository(), context.file_repository());

        (context, service)
    }

    #[tokio::test]
    pub async fn create_folder_existing_folder_returned_error() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        context
            .folder_repository()
            .create(&Folder::new(
                None,
                Some(ROOT_FOLDER_ID),
                "folder".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .create_folder(Some(ROOT_FOLDER_ID), "folder".try_into().unwrap())
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_eq!(
            FileServiceError::FolderExists {
                name: "folder".into()
            },
            actual.unwrap_err()
        );
    }

    #[tokio::test]
    pub async fn create_folder_valid_input_created_folder() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        // Act

        let actual = service
            .create_folder(Some(ROOT_FOLDER_ID), "folder".try_into().unwrap())
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_ne!(Guid::nil(), actual.unwrap());
    }

    #[tokio::test]
    pub async fn rename_folder_existing_folder_returned_error() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let folder_id = Guid::new_v4();

        context
            .folder_repository()
            .create(&Folder::new(
                Some(folder_id),
                Some(ROOT_FOLDER_ID),
                "folder 1".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context
            .folder_repository()
            .create(&Folder::new(
                None,
                Some(ROOT_FOLDER_ID),
                "folder 2".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .rename_folder(folder_id, "folder 2".try_into().unwrap())
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_eq!(
            FileServiceError::FolderExists {
                name: "folder 2".into()
            },
            actual.unwrap_err()
        );
    }

    #[tokio::test]
    pub async fn rename_folder_same_name_folder_not_changed() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let folder_id = Guid::new_v4();

        context
            .folder_repository()
            .create(&Folder::new(
                Some(folder_id),
                Some(ROOT_FOLDER_ID),
                "folder".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .rename_folder(folder_id, "folder".try_into().unwrap())
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_eq!(Ok(()), actual);
        let folder = context
            .folder_repository()
            .get_by_id(folder_id)
            .await
            .unwrap();
        assert_eq!(
            FileSystemItemName::new_unchecked("folder".to_string()),
            folder.name()
        );
    }

    #[tokio::test]
    pub async fn rename_folder_valid_input_renamed_folder() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let folder_id = Guid::new_v4();

        context
            .folder_repository()
            .create(&Folder::new(
                Some(folder_id),
                Some(ROOT_FOLDER_ID),
                "folder".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .rename_folder(folder_id, "folder 2".try_into().unwrap())
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_eq!(Ok(()), actual);
        let folder = context
            .folder_repository()
            .get_by_id(folder_id)
            .await
            .unwrap();
        assert_eq!(
            FileSystemItemName::new_unchecked("folder 2".to_string()),
            folder.name()
        );
    }

    #[tokio::test]
    pub async fn move_folder_to_nested_folder_error_returned() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let parent_folder_id = Guid::new_v4();
        let child_folder_id = Guid::new_v4();

        context
            .folder_repository()
            .create(&Folder::new(
                Some(parent_folder_id),
                Some(ROOT_FOLDER_ID),
                "parent folder".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context
            .folder_repository()
            .create(&Folder::new(
                Some(child_folder_id),
                Some(parent_folder_id),
                "nested folder".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .move_folder(parent_folder_id, Some(child_folder_id))
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_eq!(
            Err(FileServiceError::CannotMoveChildIntoInnerFolder),
            actual
        );
    }

    #[tokio::test]
    pub async fn move_folder_two_level_down_nested_folder_error_returned() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let parent_folder_id = Guid::new_v4();
        let child_folder_id1 = Guid::new_v4();
        let child_folder_id2 = Guid::new_v4();

        context
            .folder_repository()
            .create(&Folder::new(
                Some(parent_folder_id),
                Some(ROOT_FOLDER_ID),
                "parent folder".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context
            .folder_repository()
            .create(&Folder::new(
                Some(child_folder_id1),
                Some(parent_folder_id),
                "nested folder 1".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context
            .folder_repository()
            .create(&Folder::new(
                Some(child_folder_id2),
                Some(child_folder_id1),
                "nested folder 2".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .move_folder(parent_folder_id, Some(child_folder_id2))
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_eq!(
            Err(FileServiceError::CannotMoveChildIntoInnerFolder),
            actual
        );
    }

    #[tokio::test]
    pub async fn move_folder_existing_folder_error_returned() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let parent_folder_id = Guid::new_v4();
        let child_folder_id1 = Guid::new_v4();
        let child_folder_id2 = Guid::new_v4();

        context
            .folder_repository()
            .create(&Folder::new(
                Some(parent_folder_id),
                Some(ROOT_FOLDER_ID),
                "parent folder".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context
            .folder_repository()
            .create(&Folder::new(
                Some(child_folder_id1),
                Some(parent_folder_id),
                "child folder".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context
            .folder_repository()
            .create(&Folder::new(
                Some(child_folder_id2),
                Some(ROOT_FOLDER_ID),
                "child folder".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .move_folder(child_folder_id2, Some(parent_folder_id))
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_eq!(
            Err(FileServiceError::FolderExists {
                name: "child folder".to_string()
            }),
            actual
        );
    }

    #[tokio::test]
    pub async fn move_folder_valid_input_moved_folder() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let parent_folder_id1 = Guid::new_v4();
        let parent_folder_id2 = Guid::new_v4();
        let child_folder_id = Guid::new_v4();

        context
            .folder_repository()
            .create(&Folder::new(
                Some(parent_folder_id1),
                Some(ROOT_FOLDER_ID),
                "parent folder 1".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context
            .folder_repository()
            .create(&Folder::new(
                Some(parent_folder_id2),
                Some(ROOT_FOLDER_ID),
                "parent folder 2".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context
            .folder_repository()
            .create(&Folder::new(
                Some(child_folder_id),
                Some(parent_folder_id1),
                "child folder".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .move_folder(child_folder_id, Some(parent_folder_id2))
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_eq!(Ok(()), actual);
        let folder = context
            .folder_repository()
            .get_by_id(child_folder_id)
            .await
            .unwrap();
        assert_eq!(Some(parent_folder_id2), folder.parent_id());
    }

    #[tokio::test]
    pub async fn create_file_existing_file_returned_error() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        context
            .file_repository()
            .create(&File::new(
                None,
                Some(ROOT_FOLDER_ID),
                "file".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .create_file(Some(ROOT_FOLDER_ID), "file".try_into().unwrap())
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_eq!(
            FileServiceError::FileExists {
                name: "file".into()
            },
            actual.unwrap_err()
        );
    }

    #[tokio::test]
    pub async fn rename_file_existing_file_returned_error() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let file_id = Guid::new_v4();

        context
            .file_repository()
            .create(&File::new(
                Some(file_id),
                Some(ROOT_FOLDER_ID),
                "file 1".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context
            .file_repository()
            .create(&File::new(
                None,
                Some(ROOT_FOLDER_ID),
                "file 2".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .rename_file(file_id, "file 2".try_into().unwrap())
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_eq!(
            FileServiceError::FileExists {
                name: "file 2".into()
            },
            actual.unwrap_err()
        );
    }

    #[tokio::test]
    pub async fn rename_file_same_name_file_not_changed() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let file_id = Guid::new_v4();

        context
            .file_repository()
            .create(&File::new(
                Some(file_id),
                Some(ROOT_FOLDER_ID),
                "file".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .rename_file(file_id, "file".try_into().unwrap())
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_eq!(Ok(()), actual);
        let file = context.file_repository().get_by_id(file_id).await.unwrap();
        assert_eq!(
            FileSystemItemName::new_unchecked("file".to_string()),
            file.name()
        );
    }

    #[tokio::test]
    pub async fn rename_file_valid_input_renamed_file() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let file_id = Guid::new_v4();

        context
            .file_repository()
            .create(&File::new(
                Some(file_id),
                Some(ROOT_FOLDER_ID),
                "file".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .rename_file(file_id, "file 2".try_into().unwrap())
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_eq!(Ok(()), actual);
        let file = context.file_repository().get_by_id(file_id).await.unwrap();
        assert_eq!(
            FileSystemItemName::new_unchecked("file 2".to_string()),
            file.name()
        );
    }

    #[tokio::test]
    pub async fn move_file_existing_file_error_returned() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let parent_folder_id = Guid::new_v4();
        let child_file_id1 = Guid::new_v4();
        let child_file_id2 = Guid::new_v4();

        context
            .folder_repository()
            .create(&Folder::new(
                Some(parent_folder_id),
                Some(ROOT_FOLDER_ID),
                "parent folder".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context
            .file_repository()
            .create(&File::new(
                Some(child_file_id1),
                Some(parent_folder_id),
                "child file".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context
            .file_repository()
            .create(&File::new(
                Some(child_file_id2),
                Some(ROOT_FOLDER_ID),
                "child file".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .move_file(child_file_id2, Some(parent_folder_id))
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_eq!(
            Err(FileServiceError::FileExists {
                name: "child file".to_string()
            }),
            actual
        );
    }

    #[tokio::test]
    pub async fn move_file_valid_input_moved_file() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let parent_folder_id1 = Guid::new_v4();
        let parent_folder_id2 = Guid::new_v4();
        let child_file_id = Guid::new_v4();

        context
            .folder_repository()
            .create(&Folder::new(
                Some(parent_folder_id1),
                Some(ROOT_FOLDER_ID),
                "parent file 1".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context
            .folder_repository()
            .create(&Folder::new(
                Some(parent_folder_id2),
                Some(ROOT_FOLDER_ID),
                "parent file 2".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context
            .file_repository()
            .create(&File::new(
                Some(child_file_id),
                Some(parent_folder_id1),
                "child file".try_into().unwrap(),
            ))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .move_file(child_file_id, Some(parent_folder_id2))
            .await;
        context.save_changes().await.unwrap();

        // Assert

        assert_eq!(Ok(()), actual);
        let file = context
            .file_repository()
            .get_by_id(child_file_id)
            .await
            .unwrap();
        assert_eq!(Some(parent_folder_id2), file.parent_id());
    }
}
