use std::{path::PathBuf, str::FromStr, sync::Arc};

use brainy_application::settings::{
    dto::update_settings_request::UpdateSettingsRequest, settings_service::SettingsService,
};
use brainy_domain::{
    database::database_connection_manager::{
        DatabaseConnectionManager, MockDatabaseConnectionManager,
    },
    settings::{
        entities::settings::Settings, repositories::settings_repository::SettingsRepository,
        value_objects::database_location::DatabaseLocation,
    },
};
use brainy_infrastructure::settings::disk_settings_repository::DiskSettingsRepository;
use brainy_test_utils::create_test_injector;
use injector::{injector::Injector, register_scope};
use mockall::predicate::eq;
use tokio::sync::Mutex;

async fn initialize_test_injector(
    database_connection_manager: MockDatabaseConnectionManager,
) -> Injector {
    let mut injector = create_test_injector().await;

    let settings = Settings {
        ..Default::default()
    };

    injector.register_singleton(Arc::new(Mutex::new(settings)));
    injector
        .register_singleton::<dyn DatabaseConnectionManager>(Arc::new(database_connection_manager));

    register_scope!(injector, dyn SettingsRepository, DiskSettingsRepository);
    register_scope!(injector, SettingsService);

    injector
}

#[tokio::test]
pub async fn update_settings_updated_database_location_called_manager() {
    // Arrange

    let request = UpdateSettingsRequest {
        base_database_directory: Some("new path".into()),
        ..Default::default()
    };

    let mut database_connection_manager = MockDatabaseConnectionManager::new();
    database_connection_manager
        .expect_connect_to_database()
        .with(eq(DatabaseLocation::new_unchecked(
            PathBuf::from_str("new path").unwrap().join("brainy.db"),
        )))
        .returning(|_| Box::pin(async { Ok(()) }));

    let injector = initialize_test_injector(database_connection_manager).await;
    let scope = injector.start_scope();
    let service = scope.resolve::<SettingsService>().await;

    // Act & Assert

    service.update_settings(request).await.unwrap();
}

#[tokio::test]
pub async fn update_settings_did_not_update_database_location_did_not_call_manager() {
    // Arrange

    let request = UpdateSettingsRequest {
        ..Default::default()
    };

    let mut database_connection_manager = MockDatabaseConnectionManager::new();
    database_connection_manager
        .expect_connect_to_database()
        .never();

    let injector = initialize_test_injector(database_connection_manager).await;
    let scope = injector.start_scope();
    let service = scope.resolve::<SettingsService>().await;

    // Act & Assert

    service.update_settings(request).await.unwrap();
}
