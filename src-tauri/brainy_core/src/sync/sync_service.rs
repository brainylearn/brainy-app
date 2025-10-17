use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, TimeZone, Utc};
use prost::Message;
use thiserror::Error;

use crate::{
    Guid,
    backend::traits::brainy_backend_client::{BrainyBackendClient, BrainyBackendClientError},
    cells::{
        entities::{cell::Cell, review::Review},
        repositories::traits::{
            cell_repository::CellRepository, review_repository::ReviewRepository,
        },
    },
    common::{extensions::to_datetime_ext::ToDateTimeExt, repository_error::RepositoryError},
    file_system::{
        entities::{file::File, folder::Folder},
        repositories::traits::{
            file_repository::FileRepository, folder_repository::FolderRepository,
        },
        value_objects::file_system_item_name::FileSystemItemName,
    },
    generated_code::{self},
    local_configurations::repositories::traits::LocalConfigurationRepository,
    sync::{
        entities::synced_entity::{EntityType, SyncedEntity},
        repositories::traits::DeletedEntityRepository,
    },
};

const LAST_SYNC_DATE_CONFIGURATION_NAME: &str = "LAST_SYNC_DATE";
const LAST_SYNC_PAGE_CONFIGURATION_NAME: &str = "LAST_SYNC_PAGE";

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SyncError {
    #[error("{0}")]
    UnknownRepositoryError(#[from] RepositoryError),
    #[error("{0}")]
    ClientError(#[from] BrainyBackendClientError),
}

pub struct SyncService {
    folder_repository: Arc<dyn FolderRepository>,
    file_repository: Arc<dyn FileRepository>,
    cell_repository: Arc<dyn CellRepository>,
    review_repository: Arc<dyn ReviewRepository>,
    deleted_entity_repository: Arc<dyn DeletedEntityRepository>,
    local_configuration_repository: Arc<dyn LocalConfigurationRepository>,
}

impl SyncService {
    pub fn new(
        folder_repository: Arc<dyn FolderRepository>,
        file_repository: Arc<dyn FileRepository>,
        cell_repository: Arc<dyn CellRepository>,
        review_repository: Arc<dyn ReviewRepository>,
        deleted_entity_repository: Arc<dyn DeletedEntityRepository>,
        local_configuration_repository: Arc<dyn LocalConfigurationRepository>,
    ) -> Self {
        Self {
            folder_repository,
            file_repository,
            cell_repository,
            review_repository,
            deleted_entity_repository,
            local_configuration_repository,
        }
    }

    /// This function fetches ahd proccess the next fetch page, it also
    /// updates all relevant configuration for fetching.
    /// Returns true if there are more sync pages to fetch.
    pub async fn fetch_and_process_next_sync_page(
        &self,
        backend_client: &Box<dyn BrainyBackendClient>,
    ) -> Result<bool, SyncError> {
        // TODO: unit test (LWW, files and folders with same name, repetitions, deleted entities)
        // TODO: make sure to update also date time on all

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
            .get_by_name(LAST_SYNC_PAGE_CONFIGURATION_NAME)
            .await?
            .map(|conf| conf.value.parse::<u32>().unwrap())
            .unwrap_or(0);

        let result = backend_client
            .get_synced_entities_after_ordered_by_created_date(last_sync_date, last_sync_page)
            .await?;

        for synced_entity in result {
            self.process_synced_entity(synced_entity).await?;
        }

        // TODO: update configuration
        // TODO: return correct
        Ok(false)
    }

    async fn process_synced_entity(&self, synced_entity: SyncedEntity) -> Result<(), SyncError> {
        log::info!("Processing synced entity with id {} and of type {:?}", synced_entity.entity_id, synced_entity.entity_type);

        let bytes = general_purpose::STANDARD
            .decode(&synced_entity.data)
            .unwrap();

        match synced_entity.entity_type {
            EntityType::Folder => {
                let folder = generated_code::Folder::decode(&bytes[..]).unwrap();
                let entity = Folder::new_unchecked(
                    Some(synced_entity.entity_id),
                    folder.parent_id.map(|val| Guid::parse_str(&val).unwrap()),
                    FileSystemItemName::new_unchecked(folder.name),
                );
                self.folder_repository
                    .upsert_with_modified_date_if_modified_before(
                        &entity,
                        folder.modified_date.unwrap().to_datetime_utc(),
                    )
                    .await?;
            }
            EntityType::File => {
                let file = generated_code::File::decode(&bytes[..]).unwrap();
                let entity = File::new_unchecked(
                    Some(synced_entity.entity_id),
                    file.parent_id.map(|val| Guid::parse_str(&val).unwrap()),
                    FileSystemItemName::new_unchecked(file.name),
                );
                self.file_repository
                    .upsert_with_modified_date_if_modified_before(
                        &entity,
                        file.modified_date.unwrap().to_datetime_utc(),
                    )
                    .await?;
            }
            EntityType::Cell => {
                let cell = generated_code::Cell::decode(&bytes[..]).unwrap();
                let entity = Cell::new_unchecked(
                    Some(synced_entity.entity_id),
                    Guid::parse_str(&cell.file_id).unwrap(),
                    cell.content,
                    serde_json::from_str(&cell.cell_type).unwrap(),
                    cell.cell_index,
                    cell.searchable_content,
                    Vec::new(),
                );
                self.cell_repository
                    .upsert_without_repetition_and_with_modified_date_if_modified_before(
                        &entity,
                        cell.modified_date.unwrap().to_datetime_utc(),
                    )
                    .await?;
            }
            EntityType::Repetition => {
                // TODO:
            }
            EntityType::Review => {
                let review = generated_code::Review::decode(&bytes[..]).unwrap();
                let entity = Review::new(
                    Some(synced_entity.entity_id),
                    review.cell_id.map(|value| Guid::parse_str(&value).unwrap()),
                    review.study_time,
                    review.date.unwrap().to_datetime_utc(),
                    serde_json::from_str(&review.rating).unwrap(),
                );
                self.review_repository
                    .upsert_with_modified_date_if_modified_before(
                        &entity,
                        synced_entity.last_sync_date,
                    )
                    .await?;
            }
            EntityType::DeletedEntity => {
                let deleted_entity = generated_code::DeletedEntity::decode(&bytes[..]).unwrap();
                self.deleted_entity_repository
                    .apply_deleted_entity(&deleted_entity.entity_name, synced_entity.entity_id)
                    .await?;
            }
        };

        // TODO: handle file and folders with same name, (merge into same name and replace all
        // existing with new name, order by id so that the id is the same)
        // TODO: handle cells with same name

        Ok(())
    }
}
