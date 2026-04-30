use async_trait::async_trait;
use thiserror::Error;

use crate::{Guid, cells::entities::cell::CellType, common::repository_error::RepositoryError};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CellCreatorError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[async_trait]
pub trait CellCreator: Send + Sync {
    async fn create_cell(
        &self,
        file_id: Guid,
        content: String,
        cell_type: CellType,
        index: u32,
    ) -> Result<Guid, CellCreatorError>;
}
