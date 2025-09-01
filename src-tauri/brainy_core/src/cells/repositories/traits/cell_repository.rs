use async_trait::async_trait;

use crate::{cells::entities::cell::Cell, common::repository_error::RepositoryError, Guid};

#[async_trait]
pub trait CellRepository: Send + Sync {
    async fn get_file_cells_ordered_by_index(
        &self,
        file_id: Guid,
    ) -> Result<Vec<Cell>, RepositoryError>;
}
