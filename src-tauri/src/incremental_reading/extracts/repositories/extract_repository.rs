use async_trait::async_trait;

use crate::{
    Guid,
    common::repository_error::RepositoryError,
    incremental_reading::extracts::entities::extract::{Extract, ExtractStatus},
};

#[async_trait]
pub trait ExtractRepository: Send + Sync {
    async fn get_by_cell_id(&self, cell_id: Guid) -> Result<Vec<Extract>, RepositoryError>;
    async fn count_by_cell_id_and_status(
        &self,
        cell_id: Guid,
        status: &ExtractStatus,
    ) -> Result<u32, RepositoryError>;
    async fn create(&self, extract: &Extract) -> Result<(), RepositoryError>;
    async fn delete_by_id(&self, id: Guid) -> Result<(), RepositoryError>;
}
