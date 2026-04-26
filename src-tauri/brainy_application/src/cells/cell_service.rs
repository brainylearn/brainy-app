use std::sync::Arc;

use chrono::Utc;
use injector_derive::ScopeInjectable;
use thiserror::Error;

use brainy_domain::common::repository_error::RepositoryError;
use brainy_domain::{
    Guid,
    cells::{
        entities::{
            cell::{Cell, CellType},
            review::{Rating, Review},
        },
        repositories::{
            cell_repository::{CellRepository, MoveDirection},
            review_repository::ReviewRepository,
        },
        value_objects::repetition_update::RepetitionUpdate,
    },
};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CellServiceError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[derive(ScopeInjectable)]
pub struct CellService {
    cell_repository: Arc<dyn CellRepository>,
    review_repository: Arc<dyn ReviewRepository>,
}

impl CellService {
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

        self.cell_repository
            .move_cells_indices_starting_from(file_id, index, MoveDirection::Down)
            .await?;
        self.cell_repository.create(&cell).await?;

        Ok(cell.id())
    }

    pub async fn delete_by_id(&self, id: Guid) -> Result<(), CellServiceError> {
        log::info!("Deleting cell with id {id}.");
        let cell = self.cell_repository.get_by_id(id).await?;

        self.cell_repository.delete_by_id(id).await?;

        self.cell_repository
            .move_cells_indices_starting_from(cell.file_id(), cell.index(), MoveDirection::Up)
            .await?;
        Ok(())
    }

    pub async fn move_cell(&self, id: Guid, new_index: u32) -> Result<(), CellServiceError> {
        log::info!("Moving cell with id {id} to new index {new_index}.");
        let mut cell = self.cell_repository.get_by_id(id).await?;

        self.cell_repository
            .move_cells_indices_starting_from(cell.file_id(), cell.index() + 1, MoveDirection::Up)
            .await?;

        self.cell_repository
            .move_cells_indices_starting_from(cell.file_id(), new_index, MoveDirection::Down)
            .await?;

        cell.set_index(new_index);
        self.cell_repository.update(&cell).await?;

        Ok(())
    }

    pub async fn register_review(
        &self,
        repetition_update: RepetitionUpdate,
        rating: Rating,
        study_time: u32,
    ) -> Result<(), CellServiceError> {
        log::info!(
            "Registering review for repetition with id {}, and rating {rating:?}, and study time of {study_time} seconds.",
            repetition_update.id
        );

        let mut cell = self
            .cell_repository
            .get_by_id(repetition_update.cell_id)
            .await?;
        if let Some(element) = cell
            .repetitions_mut()
            .iter_mut()
            .find(|r| r.id() == repetition_update.id)
        {
            repetition_update.apply_update(element);
        } else {
            panic!("Cannot find repetition with specified id!");
        }
        self.cell_repository.update(&cell).await?;

        let review = Review::new(
            None,
            Some(cell.id()),
            study_time,
            Utc::now().to_utc(),
            rating,
        );
        self.review_repository.create(&review).await?;

        Ok(())
    }

    /// This method is used to enforce all invariants on the cell with the given id. By default all
    /// invariants should be enforced, but in some cases (like sync), you may need to
    /// call this method, to reinforce invariants that got broken in sync.
    /// The business invariants enforce in this calls are:
    /// 1. Ensuring no two cells has the same index.
    pub async fn enforce_cell_invariants_on_cell(&self, id: Guid) -> Result<(), CellServiceError> {
        log::info!("Enforcing cell invariants on cell with id {id}.");

        let cell = self.cell_repository.get_by_id(id).await?;

        if self
            .cell_repository
            .get_number_of_cells_in_file_with_index(cell.file_id(), cell.index())
            .await?
            > 1
        {
            // Ensuring that no two cells has the same index
            self.cell_repository
                .move_cells_indices_starting_from(cell.file_id(), cell.index(), MoveDirection::Down)
                .await?;
            // Updating to keep the old index.
            self.cell_repository.update(&cell).await?;
        }

        Ok(())
    }
}
