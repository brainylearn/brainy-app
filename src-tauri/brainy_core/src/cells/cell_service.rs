use std::sync::Arc;

use thiserror::Error;

use crate::{
    Guid,
    cells::{
        entities::{
            cell::{Cell, CellType},
            repetition::Repetition,
            review::Rating,
        },
        repositories::traits::cell_repository::{CellRepository, MoveDirection},
        value_objects::cell_deletion_request::CellDeletionRequest,
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

impl CellService {
    pub fn new(cell_repository: Arc<dyn CellRepository>) -> Self {
        Self { cell_repository }
    }

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
        let cell = self.cell_repository.get_by_id(id).await?;

        self.cell_repository
            .delete_by_id(CellDeletionRequest::new(id))
            .await?;

        self.cell_repository
            .move_cells_indices_starting_from(cell.file_id(), cell.index(), MoveDirection::Up)
            .await?;
        Ok(())
    }

    pub async fn move_cell(&self, id: Guid, new_index: u32) -> Result<(), CellServiceError> {
        let mut cell = self.cell_repository.get_by_id(id).await?;

        let new_index = if new_index > cell.index() {
            new_index - 1
        } else {
            new_index
        };

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

    // TODO: unit test
    pub async fn register_review(
        &self,
        new_repetition: Repetition,
        rating: Rating,
        study_time: u32,
    ) {
        // TODO: implementation, return value
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        ROOT_FOLDER_ID,
        common::{
            sqlite_repositories_context::SqliteRepositoriesContext,
            traits::repositories_context::RepositoriesContext,
        },
        file_system::entities::file::File,
    };

    use super::*;

    async fn create_test_dependencies() -> (SqliteRepositoriesContext, CellService) {
        let context = SqliteRepositoriesContext::create_testing_context().await;
        let service = CellService::new(context.cell_repository());

        (context, service)
    }

    #[tokio::test]
    pub async fn create_cell_moved_all_cells_down_and_created_cell() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let file = File::new_unchecked(None, Some(ROOT_FOLDER_ID), "test".try_into().unwrap());
        context.file_repository().create(&file).await.unwrap();

        let cells = [
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 0),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 1),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 2),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 3),
        ];

        context.cell_repository().create(&cells[0]).await.unwrap();
        context.cell_repository().create(&cells[1]).await.unwrap();
        context.cell_repository().create(&cells[2]).await.unwrap();
        context.cell_repository().create(&cells[3]).await.unwrap();

        context.save_changes().await.unwrap();

        // Act

        let actual = service
            .create_cell(file.id(), "".to_string(), CellType::Cloze, 2)
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Assert

        let actual_cells = context
            .cell_repository()
            .get_file_cells_ordered_by_index(file.id())
            .await
            .unwrap();
        assert_eq!(actual_cells[0].id(), cells[0].id());
        assert_eq!(actual_cells[1].id(), cells[1].id());
        assert_eq!(actual_cells[2].id(), actual);
        assert_eq!(actual_cells[3].id(), cells[2].id());
        assert_eq!(actual_cells[4].id(), cells[3].id());
    }

    #[tokio::test]
    pub async fn delete_by_id_moved_all_cells_up_and_deleted_cell() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let file = File::new_unchecked(None, Some(ROOT_FOLDER_ID), "test".try_into().unwrap());
        context.file_repository().create(&file).await.unwrap();

        let cells = [
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 0),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 1),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 2),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 3),
        ];

        context.cell_repository().create(&cells[0]).await.unwrap();
        context.cell_repository().create(&cells[1]).await.unwrap();
        context.cell_repository().create(&cells[2]).await.unwrap();
        context.cell_repository().create(&cells[3]).await.unwrap();

        context.save_changes().await.unwrap();

        // Act

        service.delete_by_id(cells[1].id()).await.unwrap();
        context.save_changes().await.unwrap();

        // Assert

        let actual_cells = context
            .cell_repository()
            .get_file_cells_ordered_by_index(file.id())
            .await
            .unwrap();

        assert_eq!(actual_cells[0].id(), cells[0].id());
        assert_eq!(actual_cells[0].index(), 0);

        assert_eq!(actual_cells[1].id(), cells[2].id());
        assert_eq!(actual_cells[1].index(), 1);

        assert_eq!(actual_cells[2].id(), cells[3].id());
        assert_eq!(actual_cells[2].index(), 2);
    }

    #[tokio::test]
    pub async fn move_cell_forward_moved_cell_correctly() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let file = File::new_unchecked(None, Some(ROOT_FOLDER_ID), "test".try_into().unwrap());
        context.file_repository().create(&file).await.unwrap();

        let cells = [
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 0),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 1),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 2),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 3),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 4),
        ];

        context.cell_repository().create(&cells[0]).await.unwrap();
        context.cell_repository().create(&cells[1]).await.unwrap();
        context.cell_repository().create(&cells[2]).await.unwrap();
        context.cell_repository().create(&cells[3]).await.unwrap();
        context.cell_repository().create(&cells[4]).await.unwrap();

        context.save_changes().await.unwrap();

        // Act

        service.move_cell(cells[1].id(), 3).await.unwrap();
        context.save_changes().await.unwrap();

        // Assert

        let actual_cells = context
            .cell_repository()
            .get_file_cells_ordered_by_index(file.id())
            .await
            .unwrap();

        assert_eq!(actual_cells[0].id(), cells[0].id());
        assert_eq!(actual_cells[0].index(), 0);

        assert_eq!(actual_cells[1].id(), cells[2].id());
        assert_eq!(actual_cells[1].index(), 1);

        assert_eq!(actual_cells[2].id(), cells[1].id());
        assert_eq!(actual_cells[2].index(), 2);

        assert_eq!(actual_cells[3].id(), cells[3].id());
        assert_eq!(actual_cells[3].index(), 3);

        assert_eq!(actual_cells[4].id(), cells[4].id());
        assert_eq!(actual_cells[4].index(), 4);
    }

    #[tokio::test]
    pub async fn move_cell_backward_moved_cell_correctly() {
        // Arrange

        let (mut context, service) = create_test_dependencies().await;

        let file = File::new_unchecked(None, Some(ROOT_FOLDER_ID), "test".try_into().unwrap());
        context.file_repository().create(&file).await.unwrap();

        let cells = [
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 0),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 1),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 2),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 3),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 4),
        ];

        context.cell_repository().create(&cells[0]).await.unwrap();
        context.cell_repository().create(&cells[1]).await.unwrap();
        context.cell_repository().create(&cells[2]).await.unwrap();
        context.cell_repository().create(&cells[3]).await.unwrap();
        context.cell_repository().create(&cells[4]).await.unwrap();

        context.save_changes().await.unwrap();

        // Act

        service.move_cell(cells[3].id(), 1).await.unwrap();
        context.save_changes().await.unwrap();

        // Assert

        let actual_cells = context
            .cell_repository()
            .get_file_cells_ordered_by_index(file.id())
            .await
            .unwrap();

        assert_eq!(actual_cells[0].id(), cells[0].id());
        assert_eq!(actual_cells[0].index(), 0);

        assert_eq!(actual_cells[1].id(), cells[3].id());
        assert_eq!(actual_cells[1].index(), 1);

        assert_eq!(actual_cells[2].id(), cells[1].id());
        assert_eq!(actual_cells[2].index(), 2);

        assert_eq!(actual_cells[3].id(), cells[2].id());
        assert_eq!(actual_cells[3].index(), 3);

        assert_eq!(actual_cells[4].id(), cells[4].id());
        assert_eq!(actual_cells[4].index(), 4);
    }
}
