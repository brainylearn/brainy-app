use std::sync::Arc;

use brainy_application::common::app_data_directory::AppDataDirectory;
use brainy_domain::settings::{
    entities::settings::Settings,
    repositories::settings_repository::SettingsRepository,
    value_objects::{settings_profile::SettingsProfile, theme::Theme},
};
use brainy_infrastructure::settings::disk_settings_repository::{
    DiskSettingsRepository, SETTINGS_FILE_NAME,
};
use brainy_test_utils::create_temp_directory;
use tokio::{fs::File, io::AsyncReadExt, sync::Mutex};

#[tokio::test]
pub async fn get_or_create_settings_new_settings_created_and_saved_to_disk() {
    // Arrange

    let app_data_directory = AppDataDirectory::new(create_temp_directory().await);
    let default_settings = Settings::new(
        app_data_directory.get_path().clone(),
        SettingsProfile::Default,
    );

    // Act

    DiskSettingsRepository::get_or_create_settings(&app_data_directory, default_settings)
        .await
        .unwrap();

    // Assert

    assert!(
        app_data_directory
            .get_path()
            .join(SETTINGS_FILE_NAME)
            .exists()
    );

    let mut file_content = String::new();
    File::open(app_data_directory.get_path().join(SETTINGS_FILE_NAME))
        .await
        .unwrap()
        .read_to_string(&mut file_content)
        .await
        .unwrap();

    let settings = serde_json::from_str::<Settings>(&file_content).unwrap();
    assert_eq!(
        *settings.database_location().get_path(),
        app_data_directory.get_path().join("brainy.db")
    );
    assert_eq!(settings.theme, Theme::FollowSystem);
    assert_eq!(settings.zoom_percentage, 100f64);
    assert!(settings.auto_sync);
}

#[tokio::test]
pub async fn get_or_create_settings_existing_setting_read_from_disk() {
    // Arrange

    let app_data_directory = AppDataDirectory::new(create_temp_directory().await);
    let default_settings = Settings::new(
        app_data_directory.get_path().clone(),
        SettingsProfile::Default,
    );
    let mut settings = DiskSettingsRepository::get_or_create_settings(
        &app_data_directory,
        default_settings.clone(),
    )
    .await
    .unwrap();
    settings.zoom_percentage = 1f64;

    let settings_repository = DiskSettingsRepository {
        app_data_directory: Arc::new(app_data_directory.clone()),
        settings: Arc::new(Mutex::new(settings.clone())),
    };
    settings_repository.save_settings(settings).await.unwrap();

    // Act

    let actual =
        DiskSettingsRepository::get_or_create_settings(&app_data_directory, default_settings)
            .await
            .unwrap();

    // Assert

    assert_eq!(actual.zoom_percentage, 1f64);
}
