use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, TimeZone, Utc};
use prost::Message;
use thiserror::Error;

use crate::{
    backend::{models::SyncEntityDto, traits::brainy_backend_client::{BrainyBackendClient, BrainyBackendClientError}}, cells::entities::{cell::Cell, repetition::Repetition, review::Review}, common::{extensions::into_datetime::IntoDateTime, repository_error::RepositoryError}, file_system::{
        entities::{file::File, folder::Folder}, repositories::traits::folder_repository::FolderRepository, value_objects::file_system_item_name::FileSystemItemName
    }, generated_code::{self}, local_configurations::{
        entities::LocalConfiguration,
        repositories::traits::local_configuration_repository::LocalConfigurationRepository,
    }, sync::{
        entities::{
            deleted_entity::DeletedEntity,
            synced_entity::{EntityType, SyncedEntity},
        },
        repositories::traits::sync_repository::SyncRepository,
    }, Guid
};

const LAST_SYNC_DATE_CONFIGURATION_NAME: &str = "LAST_SYNC_DATE";
const SYNC_PAGE_TO_GET_CONFIGURATION_NAME: &str = "SYNC_PAGE_TO_GET";
const LAST_SENT_SYNC_DATE_CONFIGURATION_NAME: &str = "LAST_SENT_SYNC_DATE";

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SyncError {
    #[error("{0}")]
    UnknownRepositoryError(#[from] RepositoryError),
    #[error("{0}")]
    ClientError(#[from] BrainyBackendClientError),
}

pub struct SyncService {
    backend_client: Arc<dyn BrainyBackendClient>,
    folder_repository: Arc<dyn FolderRepository>,
    sync_repository: Arc<dyn SyncRepository>,
    local_configuration_repository: Arc<dyn LocalConfigurationRepository>,
}

impl SyncService {
    pub fn new(
        backend_client: Arc<dyn BrainyBackendClient>,
        folder_repository: Arc<dyn FolderRepository>,
        sync_repository: Arc<dyn SyncRepository>,
        local_configuration_repository: Arc<dyn LocalConfigurationRepository>,
    ) -> Self {
        Self {
            backend_client,
            folder_repository,
            sync_repository,
            local_configuration_repository,
        }
    }

    /// This function fetches and proccess the next fetched page, it also
    /// updates all relevant configuration for fetching.
    /// Returns true if there are more sync pages to fetch.
    pub async fn fetch_and_process_next_sync_page(&self) -> Result<bool, SyncError> {
        let last_sync_date = self
            .local_configuration_repository
            .get_by_name(LAST_SYNC_DATE_CONFIGURATION_NAME)
            .await?
            .map(|conf| {
                DateTime::parse_from_rfc3339(&conf.value)
                    .unwrap()
                    .with_timezone(&Utc)
            })
            .unwrap_or(Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap());

        let last_sync_page = self
            .local_configuration_repository
            .get_by_name(SYNC_PAGE_TO_GET_CONFIGURATION_NAME)
            .await?
            .map(|conf| conf.value.parse::<u32>().unwrap())
            .unwrap_or(0);

        let result = self
            .backend_client
            .get_synced_entities_after_ordered_by_created_date(last_sync_date, last_sync_page)
            .await?;

        for synced_entity in result.synced_entities {
            self.process_synced_entity(synced_entity).await?;
        }

        if result.has_more {
            self.local_configuration_repository
                .upsert(&LocalConfiguration {
                    name: SYNC_PAGE_TO_GET_CONFIGURATION_NAME.to_string(),
                    value: (last_sync_page + 1).to_string(),
                })
                .await?;
        } else {
            self.local_configuration_repository
                .upsert(&LocalConfiguration {
                    name: LAST_SYNC_DATE_CONFIGURATION_NAME.to_string(),
                    value: Utc::now().to_rfc3339(),
                })
                .await?;
            self.local_configuration_repository
                .upsert(&LocalConfiguration {
                    name: SYNC_PAGE_TO_GET_CONFIGURATION_NAME.to_string(),
                    value: 0.to_string(),
                })
                .await?;
        }

        Ok(result.has_more)
    }

    async fn process_synced_entity(&self, synced_entity: SyncedEntity) -> Result<(), SyncError> {
        log::info!(
            "Processing synced entity with id {} and of type {:?}",
            synced_entity.entity_id,
            synced_entity.entity_type
        );

        let bytes = general_purpose::STANDARD
            .decode(&synced_entity.data)
            .unwrap();

        match synced_entity.entity_type {
            EntityType::Folder => {
                let folder = generated_code::Folder::decode(&bytes[..]).unwrap();
                let entity = Folder::new_unchecked(
                    synced_entity.entity_id,
                    synced_entity.created_date,
                    folder.parent_id.map(|val| Guid::parse_str(&val).unwrap()),
                    FileSystemItemName::new_unchecked(folder.name),
                );
                self.sync_repository
                    .upsert_folder_with_modified_date_if_modified_before(
                        &entity,
                        folder.modified_date.unwrap().into_datetime(),
                    )
                    .await?;
            }
            EntityType::File => {
                let file = generated_code::File::decode(&bytes[..]).unwrap();
                let entity = File::new_unchecked(
                    synced_entity.entity_id,
                    synced_entity.created_date,
                    file.parent_id.map(|val| Guid::parse_str(&val).unwrap()),
                    FileSystemItemName::new_unchecked(file.name),
                );
                self.sync_repository
                    .upsert_file_with_modified_date_if_modified_before(
                        &entity,
                        file.modified_date.unwrap().into_datetime(),
                    )
                    .await?;
            }
            EntityType::Cell => {
                let cell = generated_code::Cell::decode(&bytes[..]).unwrap();
                let entity = Cell::new_unchecked(
                    synced_entity.entity_id,
                    synced_entity.created_date,
                    Guid::parse_str(&cell.file_id).unwrap(),
                    cell.content,
                    serde_json::from_str(&cell.cell_type).unwrap(),
                    cell.index,
                    cell.searchable_content,
                    Vec::new(),
                );
                self.sync_repository
                    .upsert_cell_without_repetition_and_with_modified_date_if_modified_before(
                        &entity,
                        cell.modified_date.unwrap().into_datetime(),
                    )
                    .await?;
            }
            EntityType::Repetition => {
                let repetition = generated_code::Repetition::decode(&bytes[..]).unwrap();
                let entity = Repetition::new_unchecked(
                    synced_entity.entity_id,
                    synced_entity.created_date,
                    Guid::parse_str(&repetition.file_id).unwrap(),
                    Guid::parse_str(&repetition.cell_id).unwrap(),
                    repetition.due.unwrap().into_datetime(),
                    repetition.stability,
                    repetition.difficulty,
                    repetition.elapsed_days,
                    repetition.scheduled_days,
                    repetition.reps,
                    repetition.lapses,
                    serde_json::from_str(&repetition.state).unwrap(),
                    repetition.last_review.map(|value| value.into_datetime()),
                    repetition.additional_content,
                );
                self.sync_repository
                    .upsert_repetition_with_modified_date_if_modified_before(
                        &entity,
                        repetition.modified_date.unwrap().into_datetime(),
                    )
                    .await?;
            }
            EntityType::Review => {
                let review = generated_code::Review::decode(&bytes[..]).unwrap();
                let entity = Review::new_unchecked(
                    synced_entity.entity_id,
                    synced_entity.created_date,
                    review.cell_id.map(|value| Guid::parse_str(&value).unwrap()),
                    review.study_time,
                    review.date.unwrap().into_datetime(),
                    serde_json::from_str(&review.rating).unwrap(),
                );
                self.sync_repository
                    .upsert_review_with_modified_date_if_modified_before(
                        &entity,
                        synced_entity.last_sync_date,
                    )
                    .await?;
            }
            EntityType::DeletedEntity => {
                let deleted_entity = generated_code::DeletedEntity::decode(&bytes[..]).unwrap();
                let entity = DeletedEntity::new(
                    synced_entity.entity_id,
                    deleted_entity.entity_name,
                    synced_entity.created_date,
                    deleted_entity.deleted_date.unwrap().into_datetime(),
                );
                self.sync_repository.apply_deleted_entity(entity).await?;
            }
        };

        // TODO: handle file and folders with same name, (merge into same name and replace all
        // existing with new name, order by id so that the id is the same) (possible change current
        // with new id)
        // TODO: handle cells with same index

        Ok(())
    }

    /// Sends all entities that has changed since the last send.
    pub async fn send_unsynced_entities(&self) -> Result<(), SyncError> {
        let last_sent_sync_date = self
            .local_configuration_repository
            .get_by_name(LAST_SENT_SYNC_DATE_CONFIGURATION_NAME)
            .await?
            .map(|conf| {
                DateTime::parse_from_rfc3339(&conf.value)
                    .unwrap()
                    .with_timezone(&Utc)
            })
            .unwrap_or(Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap());

        let mut synced_entities = Vec::<SyncEntityDto>::new();

        for folder in self.folder_repository.get_all_modified_on_or_after(last_sent_sync_date).await? {
            let data = encode_to_base64(generated_code::Folder {
                name: folder.name().to_string(),
                parent_id: folder.parent_id().map(|value| value.into()),
                // TODO: modified date is necessary
                ..Default::default()
            });

            let dto = SyncEntityDto {
                entity_id: folder.id(),
                created_date: folder.created_date(),
                entity_type: EntityType::Folder,
                data
            };

            synced_entities.push(dto);
        }
        
        // TODO: exclude entities that were retrieved in fetching and processing

        Ok(())
    }
}

fn encode_to_base64<T>(message: T) -> String
where
    T: Message,
{
    let mut buffer = Vec::new();
    message.encode(&mut buffer).unwrap();
    general_purpose::STANDARD.encode(buffer)
}

#[cfg(test)]
mod tests {
    use crate::{
        ROOT_FOLDER_ID,
        backend::{
            models::SyncedEntitiesPageDto, traits::brainy_backend_client::MockBrainyBackendClient,
        },
        cells::entities::{cell::CellType, repetition::State, review::Rating},
        common::{
            extensions::into_timestamp::IntoTimestamp,
            sqlite_repositories_context::SqliteRepositoriesContext,
            traits::repositories_context::RepositoriesContext,
        },
    };

    use super::*;

    async fn create_test_dependencies() -> (SqliteRepositoriesContext, MockBrainyBackendClient) {
        let context = SqliteRepositoriesContext::create_testing_context().await;

        (context, MockBrainyBackendClient::new())
    }

    // TODO: move it to extension and use it with test and service (duplicated now)
    fn encode_to_base64<T>(message: T) -> String
    where
        T: Message,
    {
        let mut buffer = Vec::new();
        message.encode(&mut buffer).unwrap();
        general_purpose::STANDARD.encode(buffer)
    }

    #[tokio::test]
    pub async fn fetch_and_process_next_sync_page_new_entities_inserted_new_entities() {
        // Arrange

        let (mut context, mut backend_client) = create_test_dependencies().await;
        let user_id = Guid::new_v4();
        let file_id = Guid::new_v4();
        let cell_id = Guid::new_v4();
        let synced_entities: Vec<SyncedEntity> = vec![
            SyncedEntity {
                user_id,
                entity_id: Guid::new_v4(),
                entity_type: EntityType::Folder,
                created_date: Utc::now(),
                last_sync_date: Utc::now(),
                data: encode_to_base64(generated_code::Folder {
                    modified_date: Some(Utc::now().into_timestamp()),
                    name: "test".into(),
                    parent_id: Some(ROOT_FOLDER_ID.into()),
                }),
            },
            SyncedEntity {
                user_id,
                entity_id: file_id,
                entity_type: EntityType::File,
                created_date: Utc::now(),
                last_sync_date: Utc::now(),
                data: encode_to_base64(generated_code::File {
                    modified_date: Some(Utc::now().into_timestamp()),
                    name: "test".into(),
                    parent_id: Some(ROOT_FOLDER_ID.into()),
                }),
            },
            SyncedEntity {
                user_id,
                entity_id: cell_id,
                entity_type: EntityType::Cell,
                created_date: Utc::now(),
                last_sync_date: Utc::now(),
                data: encode_to_base64(generated_code::Cell {
                    modified_date: Some(Utc::now().into_timestamp()),
                    content: "content".to_string(),
                    cell_type: serde_json::to_string(&CellType::FlashCard).unwrap(),
                    index: 1,
                    searchable_content: "search".to_string(),
                    file_id: file_id.to_string(),
                }),
            },
            SyncedEntity {
                user_id,
                entity_id: Guid::new_v4(),
                entity_type: EntityType::Repetition,
                created_date: Utc::now(),
                last_sync_date: Utc::now(),
                data: encode_to_base64(generated_code::Repetition {
                    modified_date: Some(Utc::now().into_timestamp()),
                    file_id: file_id.to_string(),
                    cell_id: cell_id.to_string(),
                    due: Some(Utc::now().into_timestamp()),
                    state: serde_json::to_string(&State::Learning).unwrap(),
                    ..Default::default()
                }),
            },
            SyncedEntity {
                user_id,
                entity_id: Guid::new_v4(),
                entity_type: EntityType::Review,
                created_date: Utc::now(),
                last_sync_date: Utc::now(),
                data: encode_to_base64(generated_code::Review {
                    modified_date: Some(Utc::now().into_timestamp()),
                    cell_id: Some(cell_id.to_string()),
                    date: Some(Utc::now().into_timestamp()),
                    rating: serde_json::to_string(&Rating::Hard).unwrap(),
                    ..Default::default()
                }),
            },
        ];

        backend_client
            .expect_get_synced_entities_after_ordered_by_created_date()
            .returning(move |_, _| {
                Ok(SyncedEntitiesPageDto {
                    synced_entities: synced_entities.clone(),
                    has_more: false,
                })
            });

        // Act

        let service = SyncService::new(
            Arc::new(backend_client),
            context.folder_repository(),
            context.sync_repository(),
            context.local_configuration_repository(),
        );
        service.fetch_and_process_next_sync_page().await.unwrap();
        context.save_changes().await.unwrap();

        // Assert

        let folders = context.folder_repository().get_all_folders().await.unwrap();
        assert_eq!(2, folders.len());
        assert!(folders.iter().any(|f| f.name()
            == FileSystemItemName::new_unchecked("test".to_string())
            && f.parent_id() == Some(ROOT_FOLDER_ID)));

        let files = context.file_repository().get_all_files().await.unwrap();
        assert_eq!(1, files.len());
        assert!(files.iter().any(|f| f.name()
            == FileSystemItemName::new_unchecked("test".to_string())
            && f.parent_id() == Some(ROOT_FOLDER_ID)));

        let cells = context
            .cell_repository()
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

        let home_statistics = context
            .cell_repository()
            .get_home_statistics()
            .await
            .unwrap();
        assert_eq!(1, home_statistics.number_of_reviews);
    }

    #[tokio::test]
    pub async fn fetch_and_process_next_sync_page_existing_entity_with_older_modified_date_entity_updated()
     {
        // Arrange

        let (mut context, mut backend_client) = create_test_dependencies().await;
        let user_id = Guid::new_v4();

        let file_id = Guid::new_v4();
        let cell_id = Guid::new_v4();

        context
            .file_repository()
            .create(&File::new_unchecked(
                file_id,
                Utc::now(),
                Some(ROOT_FOLDER_ID),
                FileSystemItemName::new_unchecked("old name".to_string()),
            ))
            .await
            .unwrap();

        context
            .cell_repository()
            .create(&Cell::new_unchecked(
                cell_id,
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
        context.save_changes().await.unwrap();

        let synced_entities: Vec<SyncedEntity> = vec![
            SyncedEntity {
                user_id,
                entity_id: file_id,
                entity_type: EntityType::File,
                created_date: Utc::now(),
                last_sync_date: Utc::now(),
                data: encode_to_base64(generated_code::File {
                    modified_date: Some(Utc::now().into_timestamp()),
                    name: "new name".into(),
                    parent_id: Some(ROOT_FOLDER_ID.into()),
                }),
            },
            SyncedEntity {
                user_id,
                entity_id: cell_id,
                entity_type: EntityType::Cell,
                created_date: Utc::now(),
                last_sync_date: Utc::now(),
                data: encode_to_base64(generated_code::Cell {
                    modified_date: Some(Utc::now().into_timestamp()),
                    content: "new content".to_string(),
                    cell_type: serde_json::to_string(&CellType::FlashCard).unwrap(),
                    file_id: file_id.to_string(),
                    ..Default::default()
                }),
            },
        ];

        backend_client
            .expect_get_synced_entities_after_ordered_by_created_date()
            .returning(move |_, _| {
                Ok(SyncedEntitiesPageDto {
                    synced_entities: synced_entities.clone(),
                    has_more: false,
                })
            });

        // Act

        let service = SyncService::new(
            Arc::new(backend_client),
            context.folder_repository(),
            context.sync_repository(),
            context.local_configuration_repository(),
        );
        service.fetch_and_process_next_sync_page().await.unwrap();
        context.save_changes().await.unwrap();

        // Assert

        let files = context.file_repository().get_all_files().await.unwrap();
        assert_eq!(1, files.len());
        assert!(
            files
                .iter()
                .any(|f| f.name() == FileSystemItemName::new_unchecked("new name".to_string()))
        );

        let cells = context
            .cell_repository()
            .get_file_cells_ordered_by_index(file_id)
            .await
            .unwrap();
        assert_eq!(1, cells.len());
        assert!(cells.iter().any(|c| c.content() == "new content"));
    }

    #[tokio::test]
    pub async fn fetch_and_process_next_sync_page_existing_entity_with_newer_modified_date_entities_not_updated()
     {
        // Arrange

        let (mut context, mut backend_client) = create_test_dependencies().await;
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
                data: encode_to_base64(generated_code::File {
                    modified_date: Some(Utc::now().into_timestamp()),
                    name: "new name".into(),
                    parent_id: Some(ROOT_FOLDER_ID.into()),
                }),
            },
            SyncedEntity {
                user_id,
                entity_id: cell_id,
                entity_type: EntityType::Cell,
                created_date: Utc::now(),
                last_sync_date: Utc::now(),
                data: encode_to_base64(generated_code::Cell {
                    modified_date: Some(Utc::now().into_timestamp()),
                    content: "new content".to_string(),
                    cell_type: serde_json::to_string(&CellType::FlashCard).unwrap(),
                    file_id: file_id.to_string(),
                    ..Default::default()
                }),
            },
        ];

        context
            .file_repository()
            .create(&File::new_unchecked(
                file_id,
                Utc::now(),
                Some(ROOT_FOLDER_ID),
                FileSystemItemName::new_unchecked("old name".to_string()),
            ))
            .await
            .unwrap();

        context
            .cell_repository()
            .create(&Cell::new_unchecked(
                cell_id,
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
        context.save_changes().await.unwrap();

        backend_client
            .expect_get_synced_entities_after_ordered_by_created_date()
            .returning(move |_, _| {
                Ok(SyncedEntitiesPageDto {
                    synced_entities: synced_entities.clone(),
                    has_more: false,
                })
            });

        // Act

        let service = SyncService::new(
            Arc::new(backend_client),
            context.folder_repository(),
            context.sync_repository(),
            context.local_configuration_repository(),
        );
        service.fetch_and_process_next_sync_page().await.unwrap();
        context.save_changes().await.unwrap();

        // Assert

        let files = context.file_repository().get_all_files().await.unwrap();
        assert_eq!(1, files.len());
        assert!(
            files
                .iter()
                .any(|f| f.name() == FileSystemItemName::new_unchecked("new name".to_string()))
        );

        let cells = context
            .cell_repository()
            .get_file_cells_ordered_by_index(file_id)
            .await
            .unwrap();
        assert_eq!(1, cells.len());
        assert!(cells.iter().any(|c| c.content() == "new content"));
    }

    // TODO: test that modified date get sets correctly, configuration update, and correct return
    // value
}
