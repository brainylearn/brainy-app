use brainy_application::{
    cells::cell_service::CellService,
    file_system::file_system_service::{FileServiceError, FileSystemService},
};
use brainy_domain::{
    Guid, ROOT_FOLDER_ID,
    cells::{
        entities::cell::CellType,
        repositories::{cell_repository::CellRepository, review_repository::ReviewRepository},
    },
    file_system::{
        entities::{file::File, folder::Folder},
        models::exported_item::ExportedItemType,
        repositories::{file_repository::FileRepository, folder_repository::FolderRepository},
        value_objects::{
            file_system_item_name::FileSystemItemName, fsrs_profile_choice::FsrsProfileChoice,
        },
    },
};
use brainy_infrastructure::{
    cells::{
        sqlite_cell_repository::SqliteCellRepository,
        sqlite_review_repository::SqliteReviewRepository,
    },
    common::unit_of_work::UnitOfWorkExt,
    file_system::{
        sqlite_file_repository::SqliteFileRepository,
        sqlite_folder_repository::SqliteFolderRepository,
    },
};
use brainy_test_utils::create_test_injector;
use injector::{injector::Injector, register_scope};

async fn initialize_test_injector() -> Injector {
    let mut injector = create_test_injector().await;
    register_scope!(injector, dyn FolderRepository, SqliteFolderRepository);
    register_scope!(injector, dyn FileRepository, SqliteFileRepository);
    register_scope!(injector, dyn CellRepository, SqliteCellRepository);
    register_scope!(injector, dyn ReviewRepository, SqliteReviewRepository);
    register_scope!(injector, CellService);
    register_scope!(injector, FileSystemService);
    injector
}

#[tokio::test]
pub async fn create_folder_existing_folder_returned_error() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            None,
            Some(ROOT_FOLDER_ID),
            "folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .create_folder(Some(ROOT_FOLDER_ID), "folder".try_into().unwrap())
        .await;
    scope.save_changes().await.unwrap();

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

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    // Act

    let actual = service
        .create_folder(Some(ROOT_FOLDER_ID), "folder".try_into().unwrap())
        .await;
    scope.save_changes().await.unwrap();

    // Assert

    assert_ne!(Guid::nil(), actual.unwrap());
}

#[tokio::test]
pub async fn rename_folder_existing_folder_returned_error() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    let folder_id = Guid::new_v4();

    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(folder_id),
            Some(ROOT_FOLDER_ID),
            "folder 1".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            None,
            Some(ROOT_FOLDER_ID),
            "folder 2".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .rename_folder(folder_id, "folder 2".try_into().unwrap())
        .await;
    scope.save_changes().await.unwrap();

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

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    let folder_id = Guid::new_v4();

    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(folder_id),
            Some(ROOT_FOLDER_ID),
            "folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .rename_folder(folder_id, "folder".try_into().unwrap())
        .await;
    scope.save_changes().await.unwrap();

    // Assert

    assert_eq!(Ok(()), actual);
    let folder = scope
        .resolve::<dyn FolderRepository>()
        .await
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

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    let folder_id = Guid::new_v4();

    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(folder_id),
            Some(ROOT_FOLDER_ID),
            "folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .rename_folder(folder_id, "folder 2".try_into().unwrap())
        .await;
    scope.save_changes().await.unwrap();

    // Assert

    assert_eq!(Ok(()), actual);
    let folder = scope
        .resolve::<dyn FolderRepository>()
        .await
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

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    let parent_folder_id = Guid::new_v4();
    let child_folder_id = Guid::new_v4();

    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(parent_folder_id),
            Some(ROOT_FOLDER_ID),
            "parent folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(child_folder_id),
            Some(parent_folder_id),
            "nested folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .move_folder(parent_folder_id, Some(child_folder_id))
        .await;
    scope.save_changes().await.unwrap();

    // Assert

    assert_eq!(
        Err(FileServiceError::CannotMoveChildIntoInnerFolder),
        actual
    );
}

#[tokio::test]
pub async fn move_folder_two_level_down_nested_folder_error_returned() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    let parent_folder_id = Guid::new_v4();
    let child_folder_id1 = Guid::new_v4();
    let child_folder_id2 = Guid::new_v4();

    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(parent_folder_id),
            Some(ROOT_FOLDER_ID),
            "parent folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(child_folder_id1),
            Some(parent_folder_id),
            "nested folder 1".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(child_folder_id2),
            Some(child_folder_id1),
            "nested folder 2".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .move_folder(parent_folder_id, Some(child_folder_id2))
        .await;
    scope.save_changes().await.unwrap();

    // Assert

    assert_eq!(
        Err(FileServiceError::CannotMoveChildIntoInnerFolder),
        actual
    );
}

#[tokio::test]
pub async fn move_folder_existing_folder_error_returned() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    let parent_folder_id = Guid::new_v4();
    let child_folder_id1 = Guid::new_v4();
    let child_folder_id2 = Guid::new_v4();

    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(parent_folder_id),
            Some(ROOT_FOLDER_ID),
            "parent folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(child_folder_id1),
            Some(parent_folder_id),
            "child folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(child_folder_id2),
            Some(ROOT_FOLDER_ID),
            "child folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .move_folder(child_folder_id2, Some(parent_folder_id))
        .await;
    scope.save_changes().await.unwrap();

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

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    let parent_folder_id1 = Guid::new_v4();
    let parent_folder_id2 = Guid::new_v4();
    let child_folder_id = Guid::new_v4();

    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(parent_folder_id1),
            Some(ROOT_FOLDER_ID),
            "parent folder 1".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(parent_folder_id2),
            Some(ROOT_FOLDER_ID),
            "parent folder 2".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(child_folder_id),
            Some(parent_folder_id1),
            "child folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .move_folder(child_folder_id, Some(parent_folder_id2))
        .await;
    scope.save_changes().await.unwrap();

    // Assert

    assert_eq!(Ok(()), actual);
    let folder = scope
        .resolve::<dyn FolderRepository>()
        .await
        .get_by_id(child_folder_id)
        .await
        .unwrap();
    assert_eq!(Some(parent_folder_id2), folder.parent_id());
}

#[tokio::test]
pub async fn create_file_existing_file_returned_error() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&File::new(
            None,
            Some(ROOT_FOLDER_ID),
            "file".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .create_file(Some(ROOT_FOLDER_ID), "file".try_into().unwrap())
        .await;
    scope.save_changes().await.unwrap();

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

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    let file_id = Guid::new_v4();

    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&File::new(
            Some(file_id),
            Some(ROOT_FOLDER_ID),
            "file 1".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&File::new(
            None,
            Some(ROOT_FOLDER_ID),
            "file 2".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .rename_file(file_id, "file 2".try_into().unwrap())
        .await;
    scope.save_changes().await.unwrap();

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

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    let file_id = Guid::new_v4();

    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&File::new(
            Some(file_id),
            Some(ROOT_FOLDER_ID),
            "file".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .rename_file(file_id, "file".try_into().unwrap())
        .await;
    scope.save_changes().await.unwrap();

    // Assert

    assert_eq!(Ok(()), actual);
    let file = scope
        .resolve::<dyn FileRepository>()
        .await
        .get_by_id(file_id)
        .await
        .unwrap();
    assert_eq!(
        FileSystemItemName::new_unchecked("file".to_string()),
        file.name()
    );
}

#[tokio::test]
pub async fn rename_file_valid_input_renamed_file() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    let file_id = Guid::new_v4();

    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&File::new(
            Some(file_id),
            Some(ROOT_FOLDER_ID),
            "file".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .rename_file(file_id, "file 2".try_into().unwrap())
        .await;
    scope.save_changes().await.unwrap();

    // Assert

    assert_eq!(Ok(()), actual);
    let file = scope
        .resolve::<dyn FileRepository>()
        .await
        .get_by_id(file_id)
        .await
        .unwrap();
    assert_eq!(
        FileSystemItemName::new_unchecked("file 2".to_string()),
        file.name()
    );
}

#[tokio::test]
pub async fn move_file_existing_file_error_returned() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    let parent_folder_id = Guid::new_v4();
    let child_file_id1 = Guid::new_v4();
    let child_file_id2 = Guid::new_v4();

    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(parent_folder_id),
            Some(ROOT_FOLDER_ID),
            "parent folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&File::new(
            Some(child_file_id1),
            Some(parent_folder_id),
            "child file".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&File::new(
            Some(child_file_id2),
            Some(ROOT_FOLDER_ID),
            "child file".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .move_file(child_file_id2, Some(parent_folder_id))
        .await;
    scope.save_changes().await.unwrap();

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

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    let parent_folder_id1 = Guid::new_v4();
    let parent_folder_id2 = Guid::new_v4();
    let child_file_id = Guid::new_v4();

    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(parent_folder_id1),
            Some(ROOT_FOLDER_ID),
            "parent folder 1".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(parent_folder_id2),
            Some(ROOT_FOLDER_ID),
            "parent folder 2".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&File::new(
            Some(child_file_id),
            Some(parent_folder_id1),
            "child file".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .move_file(child_file_id, Some(parent_folder_id2))
        .await;
    scope.save_changes().await.unwrap();

    // Assert

    assert_eq!(Ok(()), actual);
    let file = scope
        .resolve::<dyn FileRepository>()
        .await
        .get_by_id(child_file_id)
        .await
        .unwrap();
    assert_eq!(Some(parent_folder_id2), file.parent_id());
}

#[tokio::test]
pub async fn convert_folder_to_exported_item_valid_input_converted_folder_and_file() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    let parent_folder_id = Guid::new_v4();
    let nested_folder_id = Guid::new_v4();
    let file_id = Guid::new_v4();

    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(parent_folder_id),
            Some(ROOT_FOLDER_ID),
            "parent folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(nested_folder_id),
            Some(parent_folder_id),
            "nested folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&File::new(
            Some(file_id),
            Some(nested_folder_id),
            "file".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();

    scope
        .resolve::<CellService>()
        .await
        .create_cell(file_id, "note 1".to_string(), CellType::Note, 0)
        .await
        .unwrap();
    scope
        .resolve::<CellService>()
        .await
        .create_cell(file_id, "note 2".to_string(), CellType::Note, 1)
        .await
        .unwrap();

    scope.save_changes().await.unwrap();

    // Act

    let actual = service
        .convert_folder_to_exported_item(parent_folder_id)
        .await
        .unwrap();

    // Assert

    assert_eq!(
        FileSystemItemName::new_unchecked("parent folder".to_string()),
        actual.name
    );
    assert_eq!(None, actual.cells);
    assert_eq!(ExportedItemType::Folder, actual.item_type);

    let actual_nested_folder = actual.children.unwrap().remove(0);
    assert_eq!(
        FileSystemItemName::new_unchecked("nested folder".to_string()),
        actual_nested_folder.name
    );
    assert_eq!(None, actual_nested_folder.cells);
    assert_eq!(ExportedItemType::Folder, actual_nested_folder.item_type);

    let actual_file = actual_nested_folder.children.unwrap().remove(0);
    assert_eq!(
        FileSystemItemName::new_unchecked("file".to_string()),
        actual_file.name
    );
    assert_eq!(ExportedItemType::File, actual_file.item_type);

    let actual_cells = actual_file.cells.unwrap();
    assert!(
        actual_cells
            .iter()
            .any(|c| c.cell_type == CellType::Note && c.content == "note 1")
    );
    assert!(
        actual_cells
            .iter()
            .any(|c| c.cell_type == CellType::Note && c.content == "note 2")
    );
}

#[tokio::test]
pub async fn import_exported_item_valid_input_imported_folders_and_files() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FileSystemService>().await;

    let parent_folder_id = Guid::new_v4();
    let nested_folder_id = Guid::new_v4();
    let file_id = Guid::new_v4();

    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(parent_folder_id),
            Some(ROOT_FOLDER_ID),
            "parent folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new(
            Some(nested_folder_id),
            Some(parent_folder_id),
            "nested folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&File::new(
            Some(file_id),
            Some(nested_folder_id),
            "file".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();

    scope
        .resolve::<CellService>()
        .await
        .create_cell(file_id, "note 1".to_string(), CellType::Note, 0)
        .await
        .unwrap();
    scope
        .resolve::<CellService>()
        .await
        .create_cell(
            file_id,
            "content<script>alert('hello')</script><button onLoad='alert'>button</button>"
                .to_string(),
            CellType::Note,
            1,
        )
        .await
        .unwrap();

    scope.save_changes().await.unwrap();

    let exported_item = service
        .convert_folder_to_exported_item(parent_folder_id)
        .await
        .unwrap();

    scope
        .resolve::<dyn FolderRepository>()
        .await
        .delete_by_id(parent_folder_id)
        .await
        .unwrap();

    scope.save_changes().await.unwrap();

    // Act

    service
        .import_exported_item(ROOT_FOLDER_ID, exported_item)
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let all_folders = scope
        .resolve::<dyn FolderRepository>()
        .await
        .get_all_folders()
        .await
        .unwrap();
    assert_eq!(3, all_folders.len());
    let actual_parent_folder = all_folders
        .iter()
        .find(|f| {
            f.name() == FileSystemItemName::new_unchecked("parent folder".to_string())
                && f.parent_id().unwrap() == ROOT_FOLDER_ID
        })
        .unwrap();
    let actual_nested_folder = all_folders
        .iter()
        .find(|f| {
            f.name() == FileSystemItemName::new_unchecked("nested folder".to_string())
                && f.parent_id().unwrap() == actual_parent_folder.id()
        })
        .unwrap();

    let all_files = scope
        .resolve::<dyn FileRepository>()
        .await
        .get_all_files()
        .await
        .unwrap();
    assert_eq!(1, all_files.len());
    let actual_file = all_files
        .iter()
        .find(|f| {
            f.name() == FileSystemItemName::new_unchecked("file".to_string())
                && f.parent_id().unwrap() == actual_nested_folder.id()
        })
        .unwrap();

    let all_cells = scope
        .resolve::<dyn CellRepository>()
        .await
        .get_file_cells_ordered_by_index(actual_file.id())
        .await
        .unwrap();
    assert_eq!(2, all_cells.len());
    assert!(
        all_cells
            .iter()
            .any(|c| c.content() == "note 1" && c.cell_type() == &CellType::Note)
    );
    // Verifying that all JS is removed.
    assert!(
        all_cells
            .iter()
            .all(|c| !c.content().contains("script") && !c.content().contains("onLoad"))
    );
}
