use brainy_domain::{
    Guid, ROOT_FOLDER_ID,
    file_system::{
        entities::{file::File, folder::Folder},
        repositories::{file_repository::FileRepository, folder_repository::FolderRepository},
        value_objects::{
            file_system_item_name::FileSystemItemName, fsrs_profile_choice::FsrsProfileChoice,
        },
    },
};
use brainy_infrastructure::{
    common::unit_of_work::UnitOfWorkExt,
    file_system::{
        sqlite_file_repository::SqliteFileRepository,
        sqlite_folder_repository::SqliteFolderRepository,
    },
};
use brainy_test_utils::create_test_injector;
use chrono::Utc;
use injector::{injector::Injector, register_scope};

async fn initialize_test_injector() -> Injector {
    let mut injector = create_test_injector().await;
    register_scope!(injector, dyn FolderRepository, SqliteFolderRepository);
    register_scope!(injector, dyn FileRepository, SqliteFileRepository);
    injector
}

#[tokio::test]
pub async fn get_all_folders_valid_input_returned_all_files() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let repository = scope.resolve::<dyn FolderRepository>().await;

    repository
        .create(&Folder::new_unchecked(
            Guid::new_v4(),
            Utc::now(),
            Utc::now(),
            Some(ROOT_FOLDER_ID),
            "folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = repository.get_all_folders().await.unwrap();

    // Assert

    assert_eq!(2, actual.len());
    assert!(
        actual
            .iter()
            .any(|f| f.name() == FileSystemItemName::new_unchecked("folder".to_string()))
    );
}

#[tokio::test]
pub async fn delete_by_id_valid_input_deleted_recursively() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let repository = scope.resolve::<dyn FolderRepository>().await;

    let parent_id = Guid::new_v4();
    repository
        .create(&Folder::new_unchecked(
            parent_id,
            Utc::now(),
            Utc::now(),
            Some(ROOT_FOLDER_ID),
            "folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    repository
        .create(&Folder::new_unchecked(
            Guid::new_v4(),
            Utc::now(),
            Utc::now(),
            Some(parent_id),
            "sub folder".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&File::new_unchecked(
            Guid::new_v4(),
            Utc::now(),
            Utc::now(),
            Some(parent_id),
            "file".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();

    scope.save_changes().await.unwrap();

    // Act

    repository.delete_by_id(parent_id).await.unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let actual = repository.get_all_folders().await.unwrap();
    // Only root should exist!
    assert_eq!(1, actual.len());
}
