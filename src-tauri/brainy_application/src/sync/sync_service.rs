use std::{collections::HashSet, sync::Arc};

use crate::cells::cell_service::{CellService, CellServiceError};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, TimeZone, Utc};
use injector_derive::ScopeInjectable;
use prost::Message;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::{
    backend::{
        clients::brainy_backend_client::{BrainyBackendClient, BrainyBackendClientError},
        models::SyncEntityDto,
    },
    common::extensions::{
        into_base64::IntoBase64, into_datetime::IntoDateTime, into_timestamp::IntoTimestamp,
    },
    generated_code::{self},
};
use brainy_domain::local_configurations::{
    entities::local_configuration::LocalConfiguration,
    repositories::local_configuration_repository::LocalConfigurationRepository,
};
use brainy_domain::{
    Guid,
    cells::{
        entities::{cell::Cell, repetition::Repetition, review::Review},
        repositories::{cell_repository::CellRepository, review_repository::ReviewRepository},
    },
    common::repository_error::RepositoryError,
    file_system::{
        entities::{file::File, folder::Folder},
        repositories::{file_repository::FileRepository, folder_repository::FolderRepository},
        value_objects::file_system_item_name::FileSystemItemName,
    },
    fsrs::{entities::fsrs_profile::FsrsProfile, repositories::fsrs_repository::FsrsRepository},
    sync::{
        entities::{
            deleted_entity::DeletedEntity,
            synced_entity::{EntityType, SyncedEntity},
        },
        repositories::sync_repository::SyncRepository,
    },
};

pub const LAST_SYNC_DATE_CONFIGURATION_NAME: &str = "LAST_SYNC_DATE";

#[derive(Error, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum SyncError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    Client(#[from] BrainyBackendClientError),
    #[error(transparent)]
    CellService(#[from] CellServiceError),
}

pub struct SyncLock(pub Mutex<()>);

#[derive(ScopeInjectable)]
pub struct SyncService {
    backend_client: Arc<dyn BrainyBackendClient>,
    folder_repository: Arc<dyn FolderRepository>,
    file_repository: Arc<dyn FileRepository>,
    cell_repository: Arc<dyn CellRepository>,
    review_repository: Arc<dyn ReviewRepository>,
    sync_repository: Arc<dyn SyncRepository>,
    local_configuration_repository: Arc<dyn LocalConfigurationRepository>,
    fsrs_repository: Arc<dyn FsrsRepository>,
    cell_service: Arc<CellService>,
    sync_lock: Arc<SyncLock>,
}

impl SyncService {
    /// Gets the entities from the backend since last sync and uploads all changed
    /// entities that were not overwritten by the server during the pull phase.
    pub async fn sync_with_backend(&self) -> Result<(), SyncError> {
        // Only allowing one sync at a time.
        let _ = self.sync_lock.0.lock().await;

        let sync_start_time = Utc::now();

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

        let mut sync_page = 0;
        // Tracks entities whose local state was overwritten by the server during the
        // pull phase. These are excluded from the subsequent push so we don't
        // immediately re-upload stale local data on top of what was just received.
        let mut entities_overwritten_by_server = HashSet::new();

        loop {
            let has_more = self
                .fetch_and_process_next_sync_page(
                    sync_page,
                    last_sync_date,
                    &mut entities_overwritten_by_server,
                )
                .await?;
            if has_more {
                sync_page += 1;
            } else {
                break;
            }
        }

        self.send_unsynced_entities_since(last_sync_date, &entities_overwritten_by_server)
            .await?;

        self.local_configuration_repository
            .upsert(&LocalConfiguration {
                name: LAST_SYNC_DATE_CONFIGURATION_NAME.to_string(),
                value: sync_start_time.to_rfc3339(),
            })
            .await?;

        log::info!("Sync is completed.");

        Ok(())
    }

    /// Fetches and processes the next sync page.
    ///
    /// Returns `true` if there are more pages to process, `false` once the last
    /// page has been handled.
    ///
    /// When the repository upsert for a received entity writes new data (i.e. the
    /// server version was newer than the local one), the entity ID is added to
    /// `entities_overwritten_by_server` so the push phase can skip it and avoid
    /// re-uploading the just-received data.
    async fn fetch_and_process_next_sync_page(
        &self,
        sync_page: u32,
        last_sync_date: DateTime<Utc>,
        entities_overwritten_by_server: &mut HashSet<Guid>,
    ) -> Result<bool, SyncError> {
        let result = self
            .backend_client
            .get_synced_entities_after_ordered_by_created_date(last_sync_date, sync_page)
            .await?;

        for synced_entity in result.synced_entities {
            let entity_id = synced_entity.entity_id;
            // `process_synced_entity` returns the number of rows affected by the
            // upsert. A non-zero value means the server version was newer than what
            // we had locally, so the local state was actually overwritten. A zero
            // return means the local version was already equal or newer and the
            // upsert was a no-op.
            let rows_affected = self.process_synced_entity(synced_entity).await?;
            if rows_affected > 0 {
                entities_overwritten_by_server.insert(entity_id);
            }
        }

        Ok(result.has_more)
    }

    async fn process_synced_entity(&self, synced_entity: SyncedEntity) -> Result<u64, SyncError> {
        log::info!(
            "Processing synced entity with id {} and of type {:?}",
            synced_entity.entity_id,
            synced_entity.entity_type
        );

        let bytes = general_purpose::STANDARD
            .decode(&synced_entity.data)
            .unwrap();

        let change_count = match synced_entity.entity_type {
            EntityType::FsrsProfile => {
                let fsrs_profile = generated_code::FsrsProfile::decode(&bytes[..]).unwrap();
                let entity = FsrsProfile::new_unchecked(
                    synced_entity.entity_id,
                    synced_entity.created_date,
                    fsrs_profile.modified_date.unwrap().into_datetime(),
                    fsrs_profile.name,
                    fsrs_profile.request_retention,
                    fsrs_profile.maximum_interval,
                    fsrs_profile.weights,
                );

                #[cfg(debug_assertions)]
                log::info!("Parsed entity {:#?}", entity);

                self.fsrs_repository
                    .upsert_with_modified_date_if_modified_before(
                        &entity,
                        fsrs_profile.modified_date.unwrap().into_datetime(),
                    )
                    .await?
            }
            EntityType::Folder => {
                let folder = generated_code::Folder::decode(&bytes[..]).unwrap();
                let entity = Folder::new_unchecked(
                    synced_entity.entity_id,
                    synced_entity.created_date,
                    folder.modified_date.unwrap().into_datetime(),
                    folder.parent_id.map(|val| Guid::parse_str(&val).unwrap()),
                    FileSystemItemName::new_unchecked(folder.name),
                    folder.fsrs_profile_id.into(),
                );

                #[cfg(debug_assertions)]
                log::info!("Parsed entity {:#?}", entity);

                self.folder_repository
                    .upsert_with_modified_date_if_modified_before(
                        &entity,
                        folder.modified_date.unwrap().into_datetime(),
                    )
                    .await?
            }
            EntityType::File => {
                let file = generated_code::File::decode(&bytes[..]).unwrap();
                let entity = File::new_unchecked(
                    synced_entity.entity_id,
                    synced_entity.created_date,
                    file.modified_date.unwrap().into_datetime(),
                    file.parent_id.map(|val| Guid::parse_str(&val).unwrap()),
                    FileSystemItemName::new_unchecked(file.name),
                    file.fsrs_profile_id.into(),
                );

                #[cfg(debug_assertions)]
                log::info!("Parsed entity {:#?}", entity);

                self.file_repository
                    .upsert_with_modified_date_if_modified_before(
                        &entity,
                        file.modified_date.unwrap().into_datetime(),
                    )
                    .await?
            }
            EntityType::Cell => {
                let cell = generated_code::Cell::decode(&bytes[..]).unwrap();
                let entity = Cell::new_unchecked(
                    synced_entity.entity_id,
                    synced_entity.created_date,
                    cell.modified_date.unwrap().into_datetime(),
                    Guid::parse_str(&cell.file_id).unwrap(),
                    cell.content,
                    serde_json::from_str(&cell.cell_type).unwrap(),
                    cell.index,
                    cell.searchable_content,
                    Vec::new(),
                );

                #[cfg(debug_assertions)]
                log::info!("Parsed entity {:#?}", entity);

                let result = self
                    .cell_repository
                    .upsert_cell_without_repetition_and_with_modified_date_if_modified_before(
                        &entity,
                        cell.modified_date.unwrap().into_datetime(),
                    )
                    .await?;
                self.cell_service
                    .enforce_cell_invariants_on_cell(synced_entity.entity_id)
                    .await?;
                result
            }
            EntityType::Repetition => {
                let repetition = generated_code::Repetition::decode(&bytes[..]).unwrap();
                let entity = Repetition::new_unchecked(
                    synced_entity.entity_id,
                    synced_entity.created_date,
                    repetition.modified_date.unwrap().into_datetime(),
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

                #[cfg(debug_assertions)]
                log::info!("Parsed entity {:#?}", entity);

                self.cell_repository
                    .upsert_repetition_with_modified_date_if_modified_before(
                        &entity,
                        repetition.modified_date.unwrap().into_datetime(),
                    )
                    .await?
            }
            EntityType::Review => {
                let review = generated_code::Review::decode(&bytes[..]).unwrap();
                let entity = Review::new_unchecked(
                    synced_entity.entity_id,
                    synced_entity.created_date,
                    review.modified_date.unwrap().into_datetime(),
                    review.cell_id.map(|value| Guid::parse_str(&value).unwrap()),
                    review.study_time,
                    review.date.unwrap().into_datetime(),
                    serde_json::from_str(&review.rating).unwrap(),
                );

                #[cfg(debug_assertions)]
                log::info!("Parsed entity {:#?}", entity);

                self.review_repository
                    .upsert_with_modified_date_if_modified_before(
                        &entity,
                        review.modified_date.unwrap().into_datetime(),
                    )
                    .await?
            }
            EntityType::DeletedEntity => {
                let deleted_entity = generated_code::DeletedEntity::decode(&bytes[..]).unwrap();
                let entity = DeletedEntity::new(
                    synced_entity.entity_id,
                    deleted_entity.entity_name,
                    synced_entity.created_date,
                    deleted_entity.deleted_date.unwrap().into_datetime(),
                );

                #[cfg(debug_assertions)]
                log::info!("Parsed entity {:#?}", entity);

                self.sync_repository.apply_deleted_entity(entity).await?
            }
        };

        Ok(change_count)
    }

    /// Sends all entities with modified date on or after `last_sync_date` to the
    /// backend, skipping any whose IDs are present in `excluded_entities`.
    async fn send_unsynced_entities_since(
        &self,
        last_sync_date: DateTime<Utc>,
        excluded_entities: &HashSet<Guid>,
    ) -> Result<(), SyncError> {
        log::info!("Sending all entities modified after date {last_sync_date} to sync.");

        let mut synced_entities = Vec::<SyncEntityDto>::new();

        for fsrs_profile in self
            .fsrs_repository
            .get_all_modified_on_or_after(last_sync_date)
            .await?
        {
            let data = generated_code::FsrsProfile {
                modified_date: Some(fsrs_profile.modified_date().into_timestamp()),
                name: fsrs_profile.name().to_string(),
                request_retention: fsrs_profile.request_retention(),
                maximum_interval: fsrs_profile.maximum_interval(),
                weights: fsrs_profile.weights().to_vec(),
            }
            .into_base64();

            let dto = SyncEntityDto {
                entity_id: fsrs_profile.id(),
                created_date: fsrs_profile.created_date(),
                entity_type: EntityType::FsrsProfile,
                data,
            };

            synced_entities.push(dto);
        }

        for folder in self
            .folder_repository
            .get_all_modified_on_or_after(last_sync_date)
            .await?
        {
            let data = generated_code::Folder {
                modified_date: Some(folder.modified_date().into_timestamp()),
                name: folder.name().to_string(),
                parent_id: folder.parent_id().map(|value| value.into()),
                fsrs_profile_id: Option::<Guid>::from(folder.fsrs_profile_choice())
                    .map(|id| id.into()),
            }
            .into_base64();

            let dto = SyncEntityDto {
                entity_id: folder.id(),
                created_date: folder.created_date(),
                entity_type: EntityType::Folder,
                data,
            };

            synced_entities.push(dto);
        }

        for file in self
            .file_repository
            .get_all_modified_on_or_after(last_sync_date)
            .await?
        {
            let data = generated_code::File {
                modified_date: Some(file.modified_date().into_timestamp()),
                name: file.name().to_string(),
                parent_id: file.parent_id().map(|value| value.into()),
                fsrs_profile_id: Option::<Guid>::from(file.fsrs_profile_choice())
                    .map(|id| id.into()),
            }
            .into_base64();

            let dto = SyncEntityDto {
                entity_id: file.id(),
                created_date: file.created_date(),
                entity_type: EntityType::File,
                data,
            };

            synced_entities.push(dto);
        }

        for cell in self
            .cell_repository
            .get_all_cells_modified_on_or_after(last_sync_date)
            .await?
        {
            let data = generated_code::Cell {
                modified_date: Some(cell.modified_date().into_timestamp()),
                index: cell.index(),
                content: cell.content().to_string(),
                file_id: cell.file_id().to_string(),
                cell_type: serde_json::to_string(&cell.cell_type()).unwrap(),
                searchable_content: cell.searchable_content().to_string(),
            }
            .into_base64();

            let dto = SyncEntityDto {
                entity_id: cell.id(),
                created_date: cell.created_date(),
                entity_type: EntityType::Cell,
                data,
            };

            synced_entities.push(dto);
        }

        for repetition in self
            .cell_repository
            .get_all_repetitions_modified_on_or_after(last_sync_date)
            .await?
        {
            let data = generated_code::Repetition {
                modified_date: Some(repetition.modified_date().into_timestamp()),
                file_id: repetition.file_id().to_string(),
                cell_id: repetition.cell_id().to_string(),
                due: Some(repetition.due().into_timestamp()),
                reps: repetition.reps(),
                stability: repetition.stability(),
                difficulty: repetition.difficulty(),
                elapsed_days: repetition.elapsed_days(),
                scheduled_days: repetition.scheduled_days(),
                lapses: repetition.lapses(),
                state: serde_json::to_string(&repetition.state()).unwrap(),
                last_review: repetition.last_review().map(|value| value.into_timestamp()),
                additional_content: repetition
                    .additional_content()
                    .map(|value| value.to_string()),
            }
            .into_base64();

            let dto = SyncEntityDto {
                entity_id: repetition.id(),
                created_date: repetition.created_date(),
                entity_type: EntityType::Repetition,
                data,
            };

            synced_entities.push(dto);
        }

        for review in self
            .review_repository
            .get_all_modified_on_or_after(last_sync_date)
            .await?
        {
            let data = generated_code::Review {
                modified_date: Some(review.modified_date.into_timestamp()),
                cell_id: review.cell_id.map(|value| value.to_string()),
                date: Some(review.date.into_timestamp()),
                rating: serde_json::to_string(&review.rating).unwrap(),
                study_time: review.study_time,
            }
            .into_base64();

            let dto = SyncEntityDto {
                entity_id: review.id,
                created_date: review.created_date,
                entity_type: EntityType::Review,
                data,
            };

            synced_entities.push(dto);
        }

        for deleted_entity in self
            .sync_repository
            .get_all_deleted_entities_on_or_after(last_sync_date)
            .await?
        {
            let data = generated_code::DeletedEntity {
                entity_name: deleted_entity.entity_name,
                deleted_date: Some(deleted_entity.deleted_date.into_timestamp()),
            }
            .into_base64();

            let dto = SyncEntityDto {
                entity_id: deleted_entity.entity_id,
                created_date: deleted_entity.entity_created_date,
                entity_type: EntityType::DeletedEntity,
                data,
            };

            synced_entities.push(dto);
        }

        synced_entities.retain(|entity| !excluded_entities.contains(&entity.entity_id));

        if !synced_entities.is_empty() {
            log::info!("Sending to backend {} entities", synced_entities.len());

            self.backend_client
                .send_synced_entities(&synced_entities)
                .await?;
        }

        Ok(())
    }
}
