use std::sync::Arc;

use brainy_application::{
    backend::{
        clients::brainy_backend_client::{BrainyBackendClient, MockBrainyBackendClient},
        models::SyncedEntitiesPageDto,
    },
    cells::cell_service::CellService,
    common::extensions::{into_base64::IntoBase64, into_timestamp::IntoTimestamp},
    generated_code,
    sync::sync_service::{LAST_SYNC_DATE_CONFIGURATION_NAME, SyncLock, SyncService},
};
use brainy_domain::{
    DEFAULT_FSRS_PROFILE_ID, Guid, ROOT_FOLDER_ID,
    cells::{
        entities::{
            cell::{Cell, CellType},
            repetition::State,
            review::Rating,
        },
        repositories::{cell_repository::CellRepository, review_repository::ReviewRepository},
    },
    file_system::{
        entities::{file::File, folder::Folder},
        repositories::{file_repository::FileRepository, folder_repository::FolderRepository},
        value_objects::{
            file_system_item_name::FileSystemItemName, fsrs_profile_choice::FsrsProfileChoice,
        },
    },
    fsrs::repositories::fsrs_repository::FsrsRepository,
    local_configurations::{
        entities::local_configuration::LocalConfiguration,
        repositories::local_configuration_repository::LocalConfigurationRepository,
    },
    sync::{
        entities::synced_entity::{EntityType, SyncedEntity},
        repositories::sync_repository::SyncRepository,
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
    fsrs::sqlite_fsrs_repository::SqliteFsrsRepository,
    local_configurations::sqlite_local_configuration_repository::SqliteLocalConfigurationRepository,
    sync::sqlite_sync_repository::SqliteSyncRepository,
};
use brainy_test_utils::create_test_injector;
use chrono::{DateTime, Duration, Utc};
use injector::{injector::Injector, register_scope};
use tokio::sync::Mutex;

async fn initialize_test_injector(backend_client: MockBrainyBackendClient) -> Injector {
    let mut injector = create_test_injector().await;
    injector.register_singleton::<dyn BrainyBackendClient>(Arc::new(backend_client));
    injector.register_singleton(Arc::new(SyncLock(Mutex::new(()))));
    register_scope!(injector, dyn FolderRepository, SqliteFolderRepository);
    register_scope!(injector, dyn FileRepository, SqliteFileRepository);
    register_scope!(injector, dyn CellRepository, SqliteCellRepository);
    register_scope!(injector, dyn ReviewRepository, SqliteReviewRepository);
    register_scope!(injector, dyn SyncRepository, SqliteSyncRepository);
    register_scope!(
        injector,
        dyn LocalConfigurationRepository,
        SqliteLocalConfigurationRepository
    );
    register_scope!(injector, dyn FsrsRepository, SqliteFsrsRepository);
    register_scope!(injector, CellService);
    register_scope!(injector, SyncService);
    injector
}

#[tokio::test]
pub async fn sync_with_backend_new_entities_from_backend_inserted_new_entities() {
    // Arrange

    let user_id = Guid::new_v4();
    let file_id = Guid::new_v4();
    let cell_id = Guid::new_v4();
    let fsrs_profile_id = Guid::new_v4();
    let file_modified_date = Utc::now() - Duration::hours(8);

    let synced_entities: Vec<SyncedEntity> = vec![
        SyncedEntity {
            user_id,
            entity_id: fsrs_profile_id,
            entity_type: EntityType::FsrsProfile,
            created_date: Utc::now(),
            last_sync_date: Utc::now(),
            data: generated_code::FsrsProfile {
                modified_date: Some(Utc::now().into_timestamp()),
                name: "test profile".into(),
                request_retention: 10f64,
                maximum_interval: 8f64,
                weights: vec![1f64],
            }
            .into_base64(),
        },
        SyncedEntity {
            user_id,
            entity_id: Guid::new_v4(),
            entity_type: EntityType::Folder,
            created_date: Utc::now(),
            last_sync_date: Utc::now(),
            data: generated_code::Folder {
                modified_date: Some(Utc::now().into_timestamp()),
                name: "test".into(),
                parent_id: Some(ROOT_FOLDER_ID.into()),
                fsrs_profile_id: None,
            }
            .into_base64(),
        },
        SyncedEntity {
            user_id,
            entity_id: file_id,
            entity_type: EntityType::File,
            created_date: Utc::now(),
            last_sync_date: Utc::now(),
            data: generated_code::File {
                modified_date: Some(file_modified_date.into_timestamp()),
                name: "test".into(),
                parent_id: Some(ROOT_FOLDER_ID.into()),
                fsrs_profile_id: Some(fsrs_profile_id.to_string()),
            }
            .into_base64(),
        },
        SyncedEntity {
            user_id,
            entity_id: cell_id,
            entity_type: EntityType::Cell,
            created_date: Utc::now(),
            last_sync_date: Utc::now(),
            data: generated_code::Cell {
                modified_date: Some(Utc::now().into_timestamp()),
                content: "content".to_string(),
                cell_type: serde_json::to_string(&CellType::FlashCard).unwrap(),
                index: 1,
                searchable_content: "search".to_string(),
                file_id: file_id.to_string(),
            }
            .into_base64(),
        },
        SyncedEntity {
            user_id,
            entity_id: Guid::new_v4(),
            entity_type: EntityType::Repetition,
            created_date: Utc::now(),
            last_sync_date: Utc::now(),
            data: generated_code::Repetition {
                modified_date: Some(Utc::now().into_timestamp()),
                file_id: file_id.to_string(),
                cell_id: cell_id.to_string(),
                due: Some(Utc::now().into_timestamp()),
                state: serde_json::to_string(&State::Learning).unwrap(),
                ..Default::default()
            }
            .into_base64(),
        },
        SyncedEntity {
            user_id,
            entity_id: Guid::new_v4(),
            entity_type: EntityType::Review,
            created_date: Utc::now(),
            last_sync_date: Utc::now(),
            data: generated_code::Review {
                modified_date: Some(Utc::now().into_timestamp()),
                cell_id: Some(cell_id.to_string()),
                date: Some(Utc::now().into_timestamp()),
                rating: serde_json::to_string(&Rating::Hard).unwrap(),
                ..Default::default()
            }
            .into_base64(),
        },
    ];

    let mut backend_client = MockBrainyBackendClient::new();
    backend_client
        .expect_get_synced_entities_after_ordered_by_created_date()
        .returning(move |_, _| {
            Ok(SyncedEntitiesPageDto {
                synced_entities: synced_entities.clone(),
                has_more: false,
            })
        });

    backend_client
        .expect_send_synced_entities()
        .returning(move |_| Ok(()));

    let injector = initialize_test_injector(backend_client).await;
    let scope = injector.start_scope();

    // Act

    scope
        .resolve::<SyncService>()
        .await
        .sync_with_backend()
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let fsrs_profiles = scope
        .resolve::<dyn FsrsRepository>()
        .await
        .get_all_fsrs_profiles()
        .await
        .unwrap();
    // Default & new profile.
    assert_eq!(2, fsrs_profiles.len());
    assert!(
        fsrs_profiles
            .iter()
            .any(|f| f.name() == "test profile" && f.request_retention() == 10f64)
    );

    let folders = scope
        .resolve::<dyn FolderRepository>()
        .await
        .get_all_folders()
        .await
        .unwrap();
    assert_eq!(2, folders.len());
    assert!(folders.iter().any(|f| f.name()
        == FileSystemItemName::new_unchecked("test".to_string())
        && f.parent_id() == Some(ROOT_FOLDER_ID)
        && f.fsrs_profile_choice() == FsrsProfileChoice::Inherit));

    let files = scope
        .resolve::<dyn FileRepository>()
        .await
        .get_all_files()
        .await
        .unwrap();
    assert_eq!(1, files.len());
    assert!(files.iter().any(|f| f.name()
        == FileSystemItemName::new_unchecked("test".to_string())
        && f.parent_id() == Some(ROOT_FOLDER_ID)
        && f.fsrs_profile_choice() == FsrsProfileChoice::Id(fsrs_profile_id)
        && (f.modified_date() - file_modified_date) <= Duration::seconds(1)));

    let cells = scope
        .resolve::<dyn CellRepository>()
        .await
        .get_file_cells_ordered_by_index(file_id)
        .await
        .unwrap();
    assert_eq!(1, cells.len());
    assert!(cells.iter().any(|c| c.file_id() == file_id
        && c.content() == "content"
        && c.cell_type() == &CellType::FlashCard
        && c.index() == 1
        && c.searchable_content() == "search"));
    assert_eq!(1, cells[0].repetitions().len());

    let home_statistics = scope
        .resolve::<dyn CellRepository>()
        .await
        .get_home_statistics()
        .await
        .unwrap();
    assert_eq!(1, home_statistics.number_of_reviews);
}

#[tokio::test]
pub async fn sync_with_backend_two_cells_with_same_index_corrected_index_and_sent_update() {
    // Arrange

    let cell_in_database_id = Guid::new_v4();
    let cell_from_sync_id = Guid::new_v4();

    let file = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        Some(ROOT_FOLDER_ID),
        "test".try_into().unwrap(),
        FsrsProfileChoice::Inherit,
    );

    let synced_entities: Vec<SyncedEntity> = vec![SyncedEntity {
        user_id: Guid::new_v4(),
        entity_id: cell_from_sync_id,
        entity_type: EntityType::Cell,
        created_date: Utc::now(),
        last_sync_date: Utc::now(),
        data: generated_code::Cell {
            modified_date: Some(Utc::now().into_timestamp()),
            content: "content".to_string(),
            cell_type: serde_json::to_string(&CellType::FlashCard).unwrap(),
            index: 1,
            searchable_content: "search".to_string(),
            file_id: file.id().to_string(),
        }
        .into_base64(),
    }];

    let mut backend_client = MockBrainyBackendClient::new();
    backend_client
        .expect_get_synced_entities_after_ordered_by_created_date()
        .returning(move |_, _| {
            Ok(SyncedEntitiesPageDto {
                synced_entities: synced_entities.clone(),
                has_more: false,
            })
        });

    // Ensuring that the new index is sent!
    backend_client
        .expect_send_synced_entities()
        .withf(move |value| value.iter().any(|s| s.entity_id == cell_in_database_id))
        .returning(move |_| Ok(()));

    let injector = initialize_test_injector(backend_client).await;
    let scope = injector.start_scope();

    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&file)
        .await
        .unwrap();
    scope
        .resolve::<dyn CellRepository>()
        .await
        .create(&Cell::new_unchecked(
            cell_in_database_id,
            Utc::now(),
            Utc::now(),
            file.id(),
            "".to_string(),
            CellType::Note,
            1,
            "".to_string(),
            Vec::new(),
        ))
        .await
        .unwrap();

    // Act

    scope
        .resolve::<SyncService>()
        .await
        .sync_with_backend()
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let cells = scope
        .resolve::<dyn CellRepository>()
        .await
        .get_file_cells_ordered_by_index(file.id())
        .await
        .unwrap();
    assert!(
        cells
            .iter()
            .any(|c| c.id() == cell_from_sync_id && c.index() == 1)
    );
    assert!(
        cells
            .iter()
            .any(|c| c.id() == cell_in_database_id && c.index() == 2)
    );
}

#[tokio::test]
pub async fn sync_with_backend_deleted_entity_from_backend_processed_correctly() {
    // Arrange

    let user_id = Guid::new_v4();
    let file_id = Guid::new_v4();

    let synced_entities: Vec<SyncedEntity> = vec![SyncedEntity {
        user_id,
        entity_id: file_id,
        entity_type: EntityType::DeletedEntity,
        created_date: Utc::now(),
        last_sync_date: Utc::now(),
        data: generated_code::DeletedEntity {
            entity_name: "files".to_string(),
            deleted_date: Some(Utc::now().into_timestamp()),
        }
        .into_base64(),
    }];

    let mut backend_client = MockBrainyBackendClient::new();

    backend_client
        .expect_get_synced_entities_after_ordered_by_created_date()
        .returning(move |_, _| {
            Ok(SyncedEntitiesPageDto {
                synced_entities: synced_entities.clone(),
                has_more: false,
            })
        });

    backend_client
        .expect_send_synced_entities()
        .returning(move |_| Ok(()));

    let injector = initialize_test_injector(backend_client).await;
    let scope = injector.start_scope();

    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&File::new_unchecked(
            file_id,
            Utc::now(),
            Utc::now(),
            Some(ROOT_FOLDER_ID),
            FileSystemItemName::new_unchecked("name".to_string()),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    scope
        .resolve::<SyncService>()
        .await
        .sync_with_backend()
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let files = scope
        .resolve::<dyn FileRepository>()
        .await
        .get_all_files()
        .await
        .unwrap();
    assert_eq!(0, files.len());
}

#[tokio::test]
pub async fn sync_with_backend_existing_entity_with_older_modified_date_local_entity_updated() {
    // Arrange

    let user_id = Guid::new_v4();
    let file_id = Guid::new_v4();
    let cell_id = Guid::new_v4();

    let synced_entities: Vec<SyncedEntity> = vec![
        SyncedEntity {
            user_id,
            entity_id: file_id,
            entity_type: EntityType::File,
            created_date: Utc::now(),
            last_sync_date: Utc::now(),
            data: generated_code::File {
                modified_date: Some(Utc::now().into_timestamp()),
                name: "new name".into(),
                parent_id: Some(ROOT_FOLDER_ID.into()),
                fsrs_profile_id: None,
            }
            .into_base64(),
        },
        SyncedEntity {
            user_id,
            entity_id: cell_id,
            entity_type: EntityType::Cell,
            created_date: Utc::now(),
            last_sync_date: Utc::now(),
            data: generated_code::Cell {
                modified_date: Some(Utc::now().into_timestamp()),
                content: "new content".to_string(),
                cell_type: serde_json::to_string(&CellType::FlashCard).unwrap(),
                file_id: file_id.to_string(),
                ..Default::default()
            }
            .into_base64(),
        },
        SyncedEntity {
            user_id,
            entity_id: DEFAULT_FSRS_PROFILE_ID,
            entity_type: EntityType::FsrsProfile,
            created_date: Utc::now(),
            last_sync_date: Utc::now(),
            data: generated_code::FsrsProfile {
                modified_date: Some(Utc::now().into_timestamp()),
                name: "new name".into(),
                request_retention: 10f64,
                maximum_interval: 8f64,
                weights: vec![1f64],
            }
            .into_base64(),
        },
    ];

    let mut backend_client = MockBrainyBackendClient::new();
    backend_client
        .expect_get_synced_entities_after_ordered_by_created_date()
        .returning(move |_, _| {
            Ok(SyncedEntitiesPageDto {
                synced_entities: synced_entities.clone(),
                has_more: false,
            })
        });

    backend_client
        .expect_send_synced_entities()
        .returning(move |_| Ok(()));

    let injector = initialize_test_injector(backend_client).await;
    let scope = injector.start_scope();

    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&File::new_unchecked(
            file_id,
            Utc::now(),
            Utc::now() - Duration::minutes(2),
            Some(ROOT_FOLDER_ID),
            FileSystemItemName::new_unchecked("old name".to_string()),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();

    scope
        .resolve::<dyn CellRepository>()
        .await
        .create(&Cell::new_unchecked(
            cell_id,
            Utc::now(),
            Utc::now() - Duration::minutes(2),
            file_id,
            "old content".to_string(),
            CellType::FlashCard,
            1,
            "".to_string(),
            Vec::new(),
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    scope
        .resolve::<SyncService>()
        .await
        .sync_with_backend()
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let files = scope
        .resolve::<dyn FileRepository>()
        .await
        .get_all_files()
        .await
        .unwrap();
    assert_eq!(1, files.len());
    assert_eq!(
        files[0].name(),
        FileSystemItemName::new_unchecked("new name".to_string())
    );

    let cells = scope
        .resolve::<dyn CellRepository>()
        .await
        .get_file_cells_ordered_by_index(file_id)
        .await
        .unwrap();
    assert_eq!(1, cells.len());
    assert!(cells.iter().any(|c| c.content() == "new content"));

    let fsrs_profiles = scope
        .resolve::<dyn FsrsRepository>()
        .await
        .get_all_fsrs_profiles()
        .await
        .unwrap();
    assert_eq!(1, fsrs_profiles.len());
    assert!(fsrs_profiles.iter().any(|c| c.name() == "new name"));
}

#[tokio::test]
pub async fn sync_with_backend_existing_entity_with_newer_modified_date_locally_entities_not_updated()
 {
    // Arrange

    let user_id = Guid::new_v4();
    let file_id = Guid::new_v4();
    let cell_id = Guid::new_v4();

    let synced_entities_modified_date = Utc::now() - Duration::seconds(1);

    let synced_entities: Vec<SyncedEntity> = vec![
        SyncedEntity {
            user_id,
            entity_id: file_id,
            entity_type: EntityType::File,
            created_date: Utc::now(),
            last_sync_date: Utc::now(),
            data: generated_code::File {
                modified_date: Some(synced_entities_modified_date.into_timestamp()),
                name: "new name".into(),
                parent_id: Some(ROOT_FOLDER_ID.into()),
                fsrs_profile_id: None,
            }
            .into_base64(),
        },
        SyncedEntity {
            user_id,
            entity_id: cell_id,
            entity_type: EntityType::Cell,
            created_date: Utc::now(),
            last_sync_date: Utc::now(),
            data: generated_code::Cell {
                modified_date: Some(synced_entities_modified_date.into_timestamp()),
                content: "new content".to_string(),
                cell_type: serde_json::to_string(&CellType::FlashCard).unwrap(),
                file_id: file_id.to_string(),
                ..Default::default()
            }
            .into_base64(),
        },
    ];

    let mut backend_client = MockBrainyBackendClient::new();
    backend_client
        .expect_get_synced_entities_after_ordered_by_created_date()
        .returning(move |_, _| {
            Ok(SyncedEntitiesPageDto {
                synced_entities: synced_entities.clone(),
                has_more: false,
            })
        });

    backend_client
        .expect_send_synced_entities()
        .returning(move |_| Ok(()));

    let injector = initialize_test_injector(backend_client).await;
    let scope = injector.start_scope();

    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&File::new_unchecked(
            file_id,
            Utc::now(),
            Utc::now(),
            Some(ROOT_FOLDER_ID),
            FileSystemItemName::new_unchecked("old name".to_string()),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();

    scope
        .resolve::<dyn CellRepository>()
        .await
        .create(&Cell::new_unchecked(
            cell_id,
            Utc::now(),
            Utc::now(),
            file_id,
            "old content".to_string(),
            CellType::FlashCard,
            1,
            "".to_string(),
            Vec::new(),
        ))
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act

    scope
        .resolve::<SyncService>()
        .await
        .sync_with_backend()
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let files = scope
        .resolve::<dyn FileRepository>()
        .await
        .get_all_files()
        .await
        .unwrap();
    assert_eq!(1, files.len());
    assert_eq!(
        files[0].name(),
        FileSystemItemName::new_unchecked("old name".to_string())
    );

    let cells = scope
        .resolve::<dyn CellRepository>()
        .await
        .get_file_cells_ordered_by_index(file_id)
        .await
        .unwrap();
    assert_eq!(1, cells.len());
    assert_eq!(cells[0].content(), "old content");
}

#[tokio::test]
pub async fn sync_with_backend_valid_input_updated_sync_date_at_end() {
    // Arrange

    let mut backend_client = MockBrainyBackendClient::new();
    backend_client
        .expect_get_synced_entities_after_ordered_by_created_date()
        .returning(move |_, _| {
            Ok(SyncedEntitiesPageDto {
                synced_entities: Vec::new(),
                has_more: false,
            })
        });

    backend_client
        .expect_send_synced_entities()
        .returning(move |_| Ok(()));

    let injector = initialize_test_injector(backend_client).await;
    let scope = injector.start_scope();

    // Act

    scope
        .resolve::<SyncService>()
        .await
        .sync_with_backend()
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Assert

    let actual_sync_date_configuration = scope
        .resolve::<dyn LocalConfigurationRepository>()
        .await
        .get_by_name(LAST_SYNC_DATE_CONFIGURATION_NAME)
        .await
        .unwrap()
        .unwrap();
    let actual_date = DateTime::parse_from_rfc3339(&actual_sync_date_configuration.value)
        .unwrap()
        .with_timezone(&Utc);

    assert!((Utc::now() - actual_date) <= Duration::seconds(5));
}

#[tokio::test]
pub async fn sync_with_backend_local_unsynced_file_sent_file() {
    // Arrange

    let file = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now(),
        Some(ROOT_FOLDER_ID),
        FileSystemItemName::new_unchecked("name".to_string()),
        FsrsProfileChoice::Inherit,
    );

    let mut backend_client = MockBrainyBackendClient::new();
    backend_client
        .expect_get_synced_entities_after_ordered_by_created_date()
        .returning(move |_, _| {
            Ok(SyncedEntitiesPageDto {
                synced_entities: Vec::new(),
                has_more: false,
            })
        });

    backend_client
        .expect_send_synced_entities()
        // The count should be 2 due to the root folder and default FSRS profile.
        .withf(move |value| value.len() == 3)
        .returning(move |_| Ok(()));

    let injector = initialize_test_injector(backend_client).await;
    let scope = injector.start_scope();

    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&file)
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act & Assert

    scope
        .resolve::<SyncService>()
        .await
        .sync_with_backend()
        .await
        .unwrap();
}

#[tokio::test]
pub async fn sync_with_backend_local_file_already_synced_did_not_send_file() {
    // Arrange

    let file = File::new_unchecked(
        Guid::new_v4(),
        Utc::now(),
        Utc::now() - Duration::seconds(10),
        Some(ROOT_FOLDER_ID),
        FileSystemItemName::new_unchecked("name".to_string()),
        FsrsProfileChoice::Inherit,
    );

    let mut backend_client = MockBrainyBackendClient::new();
    backend_client
        .expect_get_synced_entities_after_ordered_by_created_date()
        .returning(move |_, _| {
            Ok(SyncedEntitiesPageDto {
                synced_entities: Vec::new(),
                has_more: false,
            })
        });

    backend_client
        .expect_send_synced_entities()
        // The count should be 2 due to the root folder and default FSRS profile.
        .withf(move |value| value.len() == 2)
        .returning(move |_| Ok(()));

    let injector = initialize_test_injector(backend_client).await;
    let scope = injector.start_scope();

    scope
        .resolve::<dyn LocalConfigurationRepository>()
        .await
        .upsert(&LocalConfiguration {
            name: LAST_SYNC_DATE_CONFIGURATION_NAME.to_string(),
            value: Utc::now().to_rfc3339(),
        })
        .await
        .unwrap();

    scope
        .resolve::<dyn FileRepository>()
        .await
        .create(&file)
        .await
        .unwrap();
    scope.save_changes().await.unwrap();

    // Act & Assert

    scope
        .resolve::<SyncService>()
        .await
        .sync_with_backend()
        .await
        .unwrap();
}

#[tokio::test]
pub async fn sync_with_backend_overwritten_change_from_backend_did_not_send_change() {
    // Arrange

    let folder_id = Guid::new_v4();

    let synced_entities: Vec<SyncedEntity> = vec![SyncedEntity {
        user_id: Guid::new_v4(),
        entity_id: folder_id,
        entity_type: EntityType::Folder,
        created_date: Utc::now(),
        last_sync_date: Utc::now(),
        data: generated_code::Folder {
            modified_date: Some(Utc::now().into_timestamp()),
            name: "test".into(),
            parent_id: Some(ROOT_FOLDER_ID.into()),
            fsrs_profile_id: None,
        }
        .into_base64(),
    }];

    let mut backend_client = MockBrainyBackendClient::new();
    backend_client
        .expect_get_synced_entities_after_ordered_by_created_date()
        .returning(move |_, _| {
            Ok(SyncedEntitiesPageDto {
                synced_entities: synced_entities.clone(),
                has_more: false,
            })
        });

    backend_client
        .expect_send_synced_entities()
        // The count should be 2 due to the root folder, and FSRS profile, the created folder should not be sent.
        .withf(move |value| value.len() == 2)
        .returning(move |_| Ok(()));

    let injector = initialize_test_injector(backend_client).await;
    let scope = injector.start_scope();

    scope
        .resolve::<dyn FolderRepository>()
        .await
        .create(&Folder::new_unchecked(
            folder_id,
            Utc::now() - Duration::minutes(1),
            Utc::now() - Duration::minutes(1),
            None,
            FileSystemItemName::new_unchecked("test".to_string()),
            FsrsProfileChoice::Inherit,
        ))
        .await
        .unwrap();

    // Act & Assert

    scope
        .resolve::<SyncService>()
        .await
        .sync_with_backend()
        .await
        .unwrap();
    scope.save_changes().await.unwrap();
}
