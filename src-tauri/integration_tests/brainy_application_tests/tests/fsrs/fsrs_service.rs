use brainy_application::fsrs::fsrs_service::{FsrsService, FsrsServiceError};
use brainy_domain::{
    DEFAULT_FSRS_PROFILE_ID, Guid, ROOT_FOLDER_ID,
    file_system::{
        repositories::folder_repository::FolderRepository,
        value_objects::fsrs_profile_choice::FsrsProfileChoice,
    },
    fsrs::{entities::fsrs_profile::FsrsProfile, repositories::fsrs_repository::FsrsRepository},
};
use brainy_infrastructure::{
    file_system::sqlite_folder_repository::SqliteFolderRepository,
    fsrs::sqlite_fsrs_repository::SqliteFsrsRepository,
};
use brainy_test_utils::create_test_injector;
use chrono::Utc;
use injector::{injector::Injector, register_scope};

async fn initialize_test_injector() -> Injector {
    let mut injector = create_test_injector().await;
    register_scope!(injector, dyn FolderRepository, SqliteFolderRepository);
    register_scope!(injector, dyn FsrsRepository, SqliteFsrsRepository);
    register_scope!(injector, FsrsService);
    injector
}

#[tokio::test]
pub async fn delete_by_id_only_one_profile_returned_error() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let service = scope.resolve::<FsrsService>().await;

    // Act

    let result = service.delete_by_id(DEFAULT_FSRS_PROFILE_ID).await;

    // Assert

    assert_eq!(result, Err(FsrsServiceError::CannotDeleteLastProfile));
}

#[tokio::test]
pub async fn delete_by_id_delete_root_profile_updated_root_profile_and_delete_profile() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let fsrs_repository = scope.resolve::<dyn FsrsRepository>().await;
    let folder_repository = scope.resolve::<dyn FolderRepository>().await;
    let service = scope.resolve::<FsrsService>().await;

    let profile = FsrsProfile::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        "test".into(),
        1f64,
        1f64,
        vec![1f64],
    );
    fsrs_repository.create(&profile).await.unwrap();

    // Act

    service.delete_by_id(DEFAULT_FSRS_PROFILE_ID).await.unwrap();

    // Assert

    let root = folder_repository.get_by_id(ROOT_FOLDER_ID).await.unwrap();
    assert_eq!(
        root.fsrs_profile_choice().clone(),
        FsrsProfileChoice::Id(profile.id())
    );

    let all_profiles = fsrs_repository.get_all_fsrs_profiles().await.unwrap();
    assert_eq!(1, all_profiles.len());
}
