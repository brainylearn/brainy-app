use std::{path::Path, sync::Arc};

use brainy_application::backup::backup_service::{
    DATETIME_FORMAT_IN_FILE_NAMES, LAST_BACKUP_DATE_CONFIGURATION_NAME, MAX_NUMBER_OF_BACKUPS,
};
use brainy_application::{
    backup::backup_service::BackupService, common::app_data_directory::AppDataDirectory,
};
use brainy_domain::database::database_connection_manager::DatabaseConnectionManager;
use brainy_domain::{
    local_configurations::{
        entities::local_configuration::LocalConfiguration,
        repositories::local_configuration_repository::LocalConfigurationRepository,
    },
    settings::{
        entities::settings::Settings,
        repositories::settings_repository::SettingsRepository,
        value_objects::{database_location::DatabaseLocation, settings_profile::SettingsProfile},
    },
};
use brainy_infrastructure::{
    common::unit_of_work::UnitOfWorkExt,
    common::{
        db_pool::DbPool, register_scoped_tx::register_scoped_tx,
        utils::create_sqlite_pool::create_sqlite_pool,
    },
    database::sqlite_database_connection_manager::SqliteDatabaseConnectionManager,
    local_configurations::sqlite_local_configuration_repository::SqliteLocalConfigurationRepository,
    settings::disk_settings_repository::DiskSettingsRepository,
};
use brainy_test_utils::create_temp_directory;
use chrono::{DateTime, Duration, TimeZone, Utc};
use injector::{injector::Injector, register_scope};
use tokio::fs;
use tokio::sync::Mutex;

async fn initialize_test_injector() -> Injector {
    let path = create_temp_directory().await.join("brainy.db");
    create_injector_for_sqlite_path(&path).await
}

async fn create_injector_for_sqlite_path(path: &Path) -> Injector {
    let mut injector = Injector::default();

    let settings = Settings::new(create_temp_directory().await, SettingsProfile::Default);
    injector.register_singleton(Arc::new(Mutex::new(settings)));

    let app_data_directory = create_temp_directory().await;
    injector.register_singleton(Arc::new(AppDataDirectory::new(app_data_directory.clone())));

    // Must use database that is saved on disk for backups to work.
    let sqlite_pool = create_sqlite_pool(&format!("sqlite:///{}", path.to_string_lossy()))
        .await
        .unwrap();
    let database_location = DatabaseLocation::new_unchecked(app_data_directory);

    let db_pool = DbPool::new(sqlite_pool, database_location);
    injector.register_singleton(Arc::new(db_pool));
    register_scoped_tx(&mut injector);

    register_scope!(
        injector,
        dyn LocalConfigurationRepository,
        SqliteLocalConfigurationRepository
    );
    register_scope!(injector, dyn SettingsRepository, DiskSettingsRepository);
    register_scope!(
        injector,
        dyn DatabaseConnectionManager,
        SqliteDatabaseConnectionManager
    );
    register_scope!(injector, BackupService);

    injector
}

#[tokio::test]
pub async fn ensure_backup_no_backups_created_backup() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let local_configuration_repository = scope.resolve::<dyn LocalConfigurationRepository>().await;
    let service = scope.resolve::<BackupService>().await;

    // Inserting a random row in the database to see if it exists in the new backup.
    local_configuration_repository
        .upsert(&LocalConfiguration {
            name: "test_configuration".into(),
            value: "value".into(),
        })
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    service.ensure_backup().await.unwrap();

    // Assert

    let settings_repository = scope.resolve::<dyn SettingsRepository>().await;
    let settings = settings_repository.get_settings().await;
    let mut dir_entries = fs::read_dir(settings.database_directory()).await.unwrap();
    let backup = dir_entries.next_entry().await.unwrap().unwrap();
    let backup_injector = create_injector_for_sqlite_path(&backup.path()).await;

    let backup_injector_scope = backup_injector.start_scope();

    let configuration = backup_injector_scope
        .resolve::<dyn LocalConfigurationRepository>()
        .await
        .get_by_name("test_configuration")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(configuration.value, "value");
}

#[tokio::test]
pub async fn ensure_backup_two_calls_in_row_only_created_backup_once() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let local_configuration_repository = scope.resolve::<dyn LocalConfigurationRepository>().await;
    let service = scope.resolve::<BackupService>().await;

    // Act

    service.ensure_backup().await.unwrap();

    // Assert

    let settings_repository = scope.resolve::<dyn SettingsRepository>().await;
    let settings = settings_repository.get_settings().await;
    let mut dir_entries = fs::read_dir(settings.database_directory()).await.unwrap();
    dir_entries.next_entry().await.unwrap().unwrap();
    assert!(dir_entries.next_entry().await.unwrap().is_none());

    let last_backup_date = DateTime::parse_from_rfc3339(
        &local_configuration_repository
            .get_by_name(LAST_BACKUP_DATE_CONFIGURATION_NAME)
            .await
            .unwrap()
            .unwrap()
            .value,
    )
    .unwrap()
    .with_timezone(&Utc);

    assert!((Utc::now() - last_backup_date) <= Duration::seconds(5));
}

#[tokio::test]
pub async fn ensure_backup_multiple_files_deleted_oldest_file() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let database_connection_manager = scope.resolve::<dyn DatabaseConnectionManager>().await;
    let settings_repository = scope.resolve::<dyn SettingsRepository>().await;
    let settings = settings_repository.get_settings().await;
    let service = scope.resolve::<BackupService>().await;

    let mut oldest_backup_path = None;

    for i in 0..MAX_NUMBER_OF_BACKUPS {
        let path = settings.database_directory().join(format!(
            "{}.backup",
            Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, i as u32)
                .unwrap()
                .format(DATETIME_FORMAT_IN_FILE_NAMES)
        ));

        database_connection_manager
            .copy_database_to(&path)
            .await
            .unwrap();

        if oldest_backup_path.is_none() {
            oldest_backup_path = Some(path);
        }
    }

    // Act

    service.ensure_backup().await.unwrap();

    // Assert

    assert!(!oldest_backup_path.unwrap().exists());
}

#[tokio::test]
pub async fn ensure_backup_other_files_than_backup_did_not_count_them_as_backups() {
    // Arrange

    let injector = initialize_test_injector().await;
    let scope = injector.start_scope();
    let database_connection_manager = scope.resolve::<dyn DatabaseConnectionManager>().await;
    let settings_repository = scope.resolve::<dyn SettingsRepository>().await;
    let settings = settings_repository.get_settings().await;
    let service = scope.resolve::<BackupService>().await;

    let mut oldest_backup_path = None;

    for i in 0..MAX_NUMBER_OF_BACKUPS - 1 {
        let path = settings.database_directory().join(format!(
            "{}.backup",
            Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, i as u32)
                .unwrap()
                .format(DATETIME_FORMAT_IN_FILE_NAMES)
        ));

        database_connection_manager
            .copy_database_to(&path)
            .await
            .unwrap();

        if oldest_backup_path.is_none() {
            oldest_backup_path = Some(path);
        }
    }

    fs::write(settings.database_directory().join("settings.json"), "1234")
        .await
        .unwrap();
    fs::write(settings.database_directory().join("test.backup"), "1234")
        .await
        .unwrap();

    // Act

    service.ensure_backup().await.unwrap();

    // Assert

    assert!(oldest_backup_path.unwrap().exists());
}
