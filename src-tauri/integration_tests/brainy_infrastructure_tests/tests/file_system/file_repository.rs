use brainy_domain::{
    Guid, ROOT_FOLDER_ID,
    file_system::{
        entities::file::File,
        repositories::file_repository::FileRepository,
        value_objects::{
            file_system_item_name::FileSystemItemName, fsrs_profile_choice::FsrsProfileChoice,
        },
    },
};
use brainy_infrastructure::{
    common::unit_of_work::UnitOfWorkExt, file_system::sqlite_file_repository::SqliteFileRepository,
};
use brainy_test_utils::create_test_injector;
use chrono::Utc;
use injector::{injector::Injector, register_scope};

async fn initialize_test_injector() -> Injector {
    let mut injector = create_test_injector().await;
    register_scope!(injector, dyn FileRepository, SqliteFileRepository);
    injector
}

#[tokio::test]
pub async fn get_all_files_valid_input_returned_all_files() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let repository = scope.resolve::<dyn FileRepository>().await;

    repository
        .create(&File::new_unchecked(
            Guid::new_v4(),
            Utc::now(),
            Utc::now(),
            Some(ROOT_FOLDER_ID),
            "file".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    let actual = repository.get_all_files().await.unwrap();

    // Assert

    assert_eq!(1, actual.len());
    assert_eq!(
        FileSystemItemName::new_unchecked("file".to_string()),
        actual[0].name()
    );
}

#[tokio::test]
pub async fn delete_by_id_valid_input_deleted_file() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let repository = scope.resolve::<dyn FileRepository>().await;

    let file_id = Guid::new_v4();
    repository
        .create(&File::new_unchecked(
            file_id,
            Utc::now(),
            Utc::now(),
            Some(ROOT_FOLDER_ID),
            "file".try_into().unwrap(),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    repository.delete_by_id(file_id).await.unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let actual = repository.get_all_files().await.unwrap();
    assert_eq!(0, actual.len());
}
