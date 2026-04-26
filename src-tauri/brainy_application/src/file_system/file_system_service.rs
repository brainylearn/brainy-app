use std::sync::Arc;

use crate::cells::cell_service::{CellService, CellServiceError};
use brainy_domain::Guid;
use injector_derive::ScopeInjectable;
use lol_html::html_content::Element;
use lol_html::{RewriteStrSettings, element, rewrite_str};
use thiserror::Error;

use brainy_domain::cells::repositories::cell_repository::CellRepository;
use brainy_domain::common::repository_error::RepositoryError;
use brainy_domain::file_system::repositories::file_repository::FileRepository;
use brainy_domain::file_system::repositories::folder_repository::FolderRepository;
use brainy_domain::file_system::value_objects::fsrs_profile_choice::FsrsProfileChoice;
use brainy_domain::file_system::{
    entities::{file::File, folder::Folder},
    models::exported_item::{ExportedItem, ExportedItemType},
    value_objects::file_system_item_name::FileSystemItemName,
};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum FileServiceError {
    #[error("The file with the name '{name}' already exists!")]
    FileExists { name: String },
    #[error("The folder with the name '{name}' already exists!")]
    FolderExists { name: String },
    #[error("Cannot move folder to a nested folder within the current folder")]
    CannotMoveChildIntoInnerFolder,
    #[error(transparent)]
    CellService(#[from] CellServiceError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[derive(ScopeInjectable)]
pub struct FileSystemService {
    cell_service: Arc<CellService>,
    folder_repository: Arc<dyn FolderRepository>,
    file_repository: Arc<dyn FileRepository>,
    cell_repository: Arc<dyn CellRepository>,
}

impl FileSystemService {
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

        let folder = Folder::new(None, parent_id, name, FsrsProfileChoice::Inherit);
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

        if let Some(destination_folder_id) = destination_folder_id
            && self
                .is_subfolder_of(folder_id, destination_folder_id)
                .await?
        {
            return Err(FileServiceError::CannotMoveChildIntoInnerFolder);
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

        let file = File::new(None, parent_id, name, FsrsProfileChoice::Inherit);
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

    pub async fn convert_folder_to_exported_item(
        &self,
        folder_id: Guid,
    ) -> Result<ExportedItem, FileServiceError> {
        log::info!("Exporting folder with id {folder_id}.");

        let folder = self.folder_repository.get_by_id(folder_id).await?;
        let mut children = Vec::new();

        let subfolders = self.folder_repository.get_subfolders(folder_id).await?;
        for subfolder in subfolders {
            let subfolder_exported_item =
                Box::pin(self.convert_folder_to_exported_item(subfolder.id())).await?;
            children.push(subfolder_exported_item);
        }

        let files = self.file_repository.get_folder_files(folder_id).await?;
        for file in files {
            let file_exported_item = self.convert_file_to_exported_item(file.id()).await?;
            children.push(file_exported_item);
        }

        Ok(ExportedItem {
            name: folder.name(),
            item_type: ExportedItemType::Folder,
            cells: None,
            children: Some(children),
        })
    }

    pub async fn convert_file_to_exported_item(
        &self,
        file_id: Guid,
    ) -> Result<ExportedItem, FileServiceError> {
        log::info!("Exporting file with id {file_id}.");

        let file = self.file_repository.get_by_id(file_id).await?;
        let cells = self
            .cell_repository
            .get_file_cells_ordered_by_index(file_id)
            .await?;
        let exported_cells = cells.into_iter().map(|c| c.into()).collect();

        Ok(ExportedItem {
            name: file.name(),
            item_type: ExportedItemType::File,
            cells: Some(exported_cells),
            children: None,
        })
    }

    pub async fn import_exported_item(
        &self,
        import_into_folder_id: Guid,
        exported_item: ExportedItem,
    ) -> Result<(), FileServiceError> {
        match exported_item.item_type {
            ExportedItemType::File => {
                log::info!("Importing file with name {}.", exported_item.name);

                let file_id = self
                    .create_file(Some(import_into_folder_id), exported_item.name)
                    .await?;

                for (i, cell) in exported_item
                    .cells
                    .unwrap_or_default()
                    .into_iter()
                    .enumerate()
                {
                    let purified_html = purify_html(&cell.content);
                    self.cell_service
                        .create_cell(file_id, purified_html, cell.cell_type, i as u32)
                        .await?;
                }
            }
            ExportedItemType::Folder => {
                log::info!("Importing folder with name {}.", exported_item.name);

                let folder_id = self
                    .create_folder(Some(import_into_folder_id), exported_item.name)
                    .await?;

                for child in exported_item.children.unwrap_or_default() {
                    Box::pin(self.import_exported_item(folder_id, child)).await?;
                }
            }
        }

        Ok(())
    }
}

fn purify_html(html: &str) -> String {
    let handler = |el: &mut Element| {
        if el.tag_name().to_lowercase() == "script"
            || el
                .attributes()
                .iter()
                .any(|attr| attr.name().to_lowercase().starts_with("on"))
        {
            el.remove();
        }

        Ok(())
    };

    rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![element!("*", handler)],
            ..RewriteStrSettings::default()
        },
    )
    .unwrap()
}

// TODO:
