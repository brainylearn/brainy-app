use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    cells::entities::{cell::Cell, repetition::Repetition, review::Review},
    common::repository_error::RepositoryError,
    file_system::entities::{file::File, folder::Folder},
    sync::entities::deleted_entity::DeletedEntity,
};

#[async_trait]
pub trait SyncRepository: Send + Sync {
    async fn apply_deleted_entity(
        &self,
        deleted_entity: DeletedEntity,
    ) -> Result<u64, RepositoryError>;

    async fn get_all_deleted_entities_on_or_after(
        &self,
        deleted_date: DateTime<Utc>,
    ) -> Result<Vec<DeletedEntity>, RepositoryError>;

    // TODO: Should these be here?
    async fn upsert_folder_with_modified_date_if_modified_before(
        &self,
        folder: &Folder,
        modified_date: DateTime<Utc>,
    ) -> Result<u64, RepositoryError>;

    async fn upsert_file_with_modified_date_if_modified_before(
        &self,
        file: &File,
        modified_date: DateTime<Utc>,
    ) -> Result<u64, RepositoryError>;

    async fn upsert_cell_without_repetition_and_with_modified_date_if_modified_before(
        &self,
        cell: &Cell,
        modified_date: DateTime<Utc>,
    ) -> Result<u64, RepositoryError>;

    async fn upsert_repetition_with_modified_date_if_modified_before(
        &self,
        repetition: &Repetition,
        modified_date: DateTime<Utc>,
    ) -> Result<u64, RepositoryError>;

    async fn upsert_review_with_modified_date_if_modified_before(
        &self,
        review: &Review,
        modified_date: DateTime<Utc>,
    ) -> Result<u64, RepositoryError>;
}
