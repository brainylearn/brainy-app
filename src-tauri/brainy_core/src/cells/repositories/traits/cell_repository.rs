use async_trait::async_trait;

use crate::{
    Guid,
    cells::{
        entities::{cell::Cell, repetition::Repetition},
        value_objects::cell_deletion_request::CellDeletionRequest,
    },
    common::repository_error::RepositoryError,
};

#[derive(PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
}

#[async_trait]
pub trait CellRepository: Send + Sync {
    async fn get_by_id(&self, id: Guid) -> Result<Cell, RepositoryError>;

    async fn get_file_cells_ordered_by_index(
        &self,
        file_id: Guid,
    ) -> Result<Vec<Cell>, RepositoryError>;

    async fn create(&self, cell: &Cell) -> Result<(), RepositoryError>;
    async fn update(&self, cell: &Cell) -> Result<(), RepositoryError>;

    /// Moves all the indicies of cells up or down based on the given direction.
    /// The cells moved must belong to the file given and must have an index
    /// greater than or equal to the given value.
    async fn move_cells_indices_starting_from(
        &self,
        file_id: Guid,
        start_index: u32,
        direction: MoveDirection,
    ) -> Result<(), RepositoryError>;

    async fn delete_by_id(&self, id: CellDeletionRequest) -> Result<(), RepositoryError>;

    async fn search_cells(&self, search_text: &str) -> Result<Vec<Cell>, RepositoryError>;

    async fn get_file_repetitions(&self, file_id: Guid)
    -> Result<Vec<Repetition>, RepositoryError>;
}
