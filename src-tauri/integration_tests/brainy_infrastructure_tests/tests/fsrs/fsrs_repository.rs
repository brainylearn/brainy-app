use brainy_domain::{
    DEFAULT_FSRS_PROFILE_ID, Guid,
    fsrs::{entities::fsrs_profile::FsrsProfile, repositories::fsrs_repository::FsrsRepository},
};
use brainy_infrastructure::fsrs::sqlite_fsrs_repository::SqliteFsrsRepository;
use brainy_test_utils::create_test_injector;
use chrono::Utc;
use injector::{injector::Injector, register_scope};

async fn initialize_test_injector() -> Injector {
    let mut injector = create_test_injector().await;
    register_scope!(injector, dyn FsrsRepository, SqliteFsrsRepository);
    injector
}

#[tokio::test]
pub async fn get_by_id_valid_input_returned_profile() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let fsrs_repository = scope.resolve::<dyn FsrsRepository>().await;

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

    let actual = fsrs_repository.get_by_id(profile.id()).await.unwrap();

    // Assert

    assert_eq!("test".to_string(), actual.name());
    assert_eq!(1f64, actual.request_retention());
}

#[tokio::test]
pub async fn get_all_fsrs_profiles_valid_input_returned_all_profiles() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let fsrs_repository = scope.resolve::<dyn FsrsRepository>().await;

    let profile1 = FsrsProfile::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        "test".into(),
        1f64,
        1f64,
        vec![1f64],
    );
    fsrs_repository.create(&profile1).await.unwrap();

    let profile2 = FsrsProfile::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        "test".into(),
        1f64,
        1f64,
        vec![1f64],
    );
    fsrs_repository.create(&profile2).await.unwrap();

    // Act

    let actual = fsrs_repository.get_all_fsrs_profiles().await.unwrap();

    // Assert

    assert_eq!(3, actual.len());
    assert!(actual.iter().any(|item| item.id() == profile1.id()));
    assert!(actual.iter().any(|item| item.id() == profile2.id()));
    // Default profile, always created.
    assert!(
        actual
            .iter()
            .any(|item| item.id() == DEFAULT_FSRS_PROFILE_ID)
    );
}

#[tokio::test]
pub async fn update_valid_input_updated_profile() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let fsrs_repository = scope.resolve::<dyn FsrsRepository>().await;

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

    let updated_profile = FsrsProfile::new_unchecked(
        profile.id(),
        Utc::now(),
        Utc::now(),
        "new name".into(),
        2f64,
        2f64,
        vec![1f64],
    );

    // Act

    fsrs_repository.update(&updated_profile).await.unwrap();

    // Assert

    let actual = fsrs_repository.get_by_id(profile.id()).await.unwrap();
    assert_eq!("new name".to_string(), actual.name());
    assert_eq!(2f64, actual.request_retention());
}
