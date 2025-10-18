use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{cells::entities::{cell::Cell, repetition::Repetition, review::Review}, common::repository_error::RepositoryError, file_system::entities::{file::File, folder::Folder}, Guid};

#[async_trait]
pub trait SyncRepository: Send + Sync {
    async fn apply_deleted_entity(
        &self,
        entity_name: &str,
        entity_id: Guid,
        delete_date: DateTime<Utc>,
    ) -> Result<(), RepositoryError>;

    async fn upsert_folder_with_modified_date_if_modified_before(
        &self,
        folder: &Folder,
        modified_date: DateTime<Utc>,
    ) -> Result<(), RepositoryError>;

    async fn upsert_file_with_modified_date_if_modified_before(
        &self,
        file: &File,
        modified_date:DateTime<Utc>,
    ) -> Result<(), RepositoryError>;

    async fn upsert_cell_without_repetition_and_with_modified_date_if_modified_before(
        &self,
        cell: &Cell,
        modified_date: DateTime<Utc>,
    ) -> Result<(), RepositoryError>;

    async fn upsert_repetition_with_modified_date_if_modified_before(
        &self,
        repetition: &Repetition,
        modified_date: DateTime<Utc>,
    ) -> Result<(), RepositoryError>;

    async fn upsert_review_with_modified_date_if_modified_before(
        &self,
        review: &Review,
        modified_date: DateTime<Utc>,
    ) -> Result<(), RepositoryError>;
}
