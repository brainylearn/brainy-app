use async_trait::async_trait;

use crate::{
    Guid, common::repository_error::RepositoryError,
    incremental_reading::extracts::entities::extract::Extract,
};

#[async_trait]
pub trait ExtractRepository: Send + Sync {
    async fn get_by_cell_id(&self, cell_id: Guid) -> Result<Vec<Extract>, RepositoryError>;
    async fn create(&self, extract: &Extract) -> Result<(), RepositoryError>;
    async fn delete_by_id(&self, id: Guid) -> Result<(), RepositoryError>;
    async fn update_inner_html(&self, id: Guid, inner_html: String) -> Result<(), RepositoryError>;
}
