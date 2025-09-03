use std::sync::Arc;

use thiserror::Error;

use crate::{
    Guid,
    cells::{
        entities::cell::{Cell, CellType},
        repositories::traits::cell_repository::{CellRepository, MoveDirection},
    },
    common::repository_error::RepositoryError,
};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CellServiceError {
    #[error("{0}")]
    UnknownRepositoryError(#[from] RepositoryError),
}

pub struct CellService {
    cell_repository: Arc<dyn CellRepository>,
}

// TODO: make sure to copy from cell service!
impl CellService {
    pub fn new(cell_repository: Arc<dyn CellRepository>) -> Self {
        Self { cell_repository }
    }

    // TODO: unit test
    pub async fn create_cell(
        &self,
        file_id: Guid,
        content: String,
        cell_type: CellType,
        index: u32,
    ) -> Result<Guid, CellServiceError> {
        log::info!(
            "Creating cell on file with id {file_id}, and cell type {cell_type}, and index {index}"
        );

        let cell = Cell::new(None, file_id, content, cell_type, index);

        // TODO: repetitions

        self.cell_repository
            .move_cells_indices_starting_from(file_id, index, MoveDirection::Down)
            .await?;
        self.cell_repository.create(&cell).await?;

        Ok(cell.id())
    }

    pub async fn delete_by_id(&self, id: Guid) -> Result<(), CellServiceError> {
        let cell = self.cell_repository.get_by_id(id).await?;

        // TODO: repetitions

        self.cell_repository.delete_by_id(id).await?;

        self.cell_repository
            .move_cells_indices_starting_from(cell.file_id(), cell.index(), MoveDirection::Up)
            .await?;
        Ok(())
    }
}
