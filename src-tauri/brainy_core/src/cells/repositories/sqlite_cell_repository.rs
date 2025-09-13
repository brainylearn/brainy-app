use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    cells::{
        entities::cell::Cell,
        repositories::{
            sqlite_rows::cell_row::{convert_rows_to_cells, CellRow}, traits::cell_repository::{CellRepository, MoveDirection}
        },
        value_objects::cell_deletion_request::CellDeletionRequest,
    }, common::repository_error::RepositoryError, Guid
};

pub struct SqliteCellRepository {
    pool: Arc<SqlitePool>,
    tx: Arc<Mutex<Transaction<'static, Sqlite>>>,
}

impl SqliteCellRepository {
    pub fn new(pool: Arc<SqlitePool>, tx: Arc<Mutex<Transaction<'static, Sqlite>>>) -> Self {
        Self { pool, tx }
    }
}

// TODO: update unit tests for repetitions, all methods!
#[async_trait]
impl CellRepository for SqliteCellRepository {
    async fn get_by_id(&self, id: Guid) -> Result<Cell, RepositoryError> {
        let rows = sqlx::query_as!(
            CellRow,
            r#"SELECT
                cell.id as "cell_id: _",
                cell.file_id as "cell_file_id: _",
                cell.content as cell_content,
                cell.cell_index as "cell_index: _",
                cell.cell_type as "cell_type: _",
                cell.searchable_content as cell_searchable_content,

                repetition.id as "repetition_id: _",
                repetition.file_id as "repetition_file_id: _",
                repetition.cell_id as "repetition_cell_id: _",
                repetition.due as "repetition_due: _",
                repetition.stability as "repetition_stability: _",
                repetition.difficulty as "repetition_difficulty: _",
                repetition.elapsed_days as "repetition_elapsed_days: _",
                repetition.scheduled_days as "repetition_scheduled_days",
                repetition.reps as "repetition_reps: _",
                repetition.lapses as "repetition_lapses: _",
                repetition.state as "repetition_state: _",
                repetition.last_review as "repetition_last_review: _",
                repetition.additional_content as "repetition_additional_content: _"

            FROM cells As cell
            LEFT JOIN repetitions AS repetition ON repetition.cell_id = cell.id
            WHERE cell.id = $1"#,
            id
        )
        .fetch_all(&*self.pool)
        .await;

        match rows {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(rows) => {
                // Should be a single cell in list.
                let cell = convert_rows_to_cells(rows).remove(0);
                Ok(cell)
            },
        }
    }

    async fn get_file_cells_ordered_by_index(
        &self,
        file_id: Guid,
    ) -> Result<Vec<Cell>, RepositoryError> {
        let rows = sqlx::query_as!(
            CellRow,
            r#"SELECT
                cell.id as "cell_id: _",
                cell.file_id as "cell_file_id: _",
                cell.content as cell_content,
                cell.cell_index as "cell_index: _",
                cell.cell_type as "cell_type: _",
                cell.searchable_content as cell_searchable_content,

                repetition.id as "repetition_id: _",
                repetition.file_id as "repetition_file_id: _",
                repetition.cell_id as "repetition_cell_id: _",
                repetition.due as "repetition_due: _",
                repetition.stability as "repetition_stability: _",
                repetition.difficulty as "repetition_difficulty: _",
                repetition.elapsed_days as "repetition_elapsed_days: _",
                repetition.scheduled_days as "repetition_scheduled_days",
                repetition.reps as "repetition_reps: _",
                repetition.lapses as "repetition_lapses: _",
                repetition.state as "repetition_state: _",
                repetition.last_review as "repetition_last_review: _",
                repetition.additional_content as "repetition_additional_content: _"

            FROM cells As cell
            LEFT JOIN repetitions AS repetition ON repetition.cell_id = cell.id

            WHERE cell.file_id = $1
            ORDER BY cell.cell_index"#,
            file_id
        )
        .fetch_all(&*self.pool)
        .await;

        match rows {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(rows) => {
                let cells = convert_rows_to_cells(rows);
                Ok(cells)
            },
        }
    }

    async fn create(&self, cell: &Cell) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let id = cell.id();
        let content = cell.content();
        let cell_type = cell.cell_type();
        let file_id = cell.file_id();
        let index = cell.index();
        let searchable_content = cell.searchable_content();

        let result = sqlx::query!(
            r#"INSERT INTO
                cells(id, content, cell_type, cell_index, file_id, searchable_content)
                VALUES ($1, $2, $3, $4, $5, $6)"#,
            id,
            content,
            cell_type,
            index,
            file_id,
            searchable_content
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn update(&self, cell: &Cell) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let id = cell.id();
        let content = cell.content();
        let cell_type = cell.cell_type();
        let file_id = cell.file_id();
        let index = cell.index();
        let searchable_content = cell.searchable_content();

        let result = sqlx::query!(
            r#"UPDATE cells
                SET id = $1,
                    file_id = $2,
                    content = $3,
                    cell_type = $4,
                    cell_index = $5,
                    searchable_content = $6
                WHERE id = $1"#,
            id,
            file_id,
            content,
            cell_type,
            index,
            searchable_content
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn move_cells_indices_starting_from(
        &self,
        file_id: Guid,
        start_index: u32,
        direction: MoveDirection,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let increase_value = if direction == MoveDirection::Up {
            -1
        } else {
            1
        };

        let result = sqlx::query!(
            r#"UPDATE cells
                SET cell_index = cell_index + $1
                WHERE file_id = $2 AND cell_index >= $3"#,
            increase_value,
            file_id,
            start_index
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn delete_by_id(
        &self,
        deletion_request: CellDeletionRequest,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let cell_id = deletion_request.id();
        let result = sqlx::query!(r#"DELETE FROM cells WHERE id = $1"#, cell_id,)
            .execute(&mut *tx)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn search_cells(&self, search_text: &str) -> Result<Vec<Cell>, RepositoryError> {
        let pattern = format!("%{}%", search_text.to_lowercase());

        let rows = sqlx::query_as!(
            CellRow,
            r#"SELECT
                cell.id as "cell_id: _",
                cell.file_id as "cell_file_id: _",
                cell.content as cell_content,
                cell.cell_index as "cell_index: _",
                cell.cell_type as "cell_type: _",
                cell.searchable_content as cell_searchable_content,

                repetition.id as "repetition_id: _",
                repetition.file_id as "repetition_file_id: _",
                repetition.cell_id as "repetition_cell_id: _",
                repetition.due as "repetition_due: _",
                repetition.stability as "repetition_stability: _",
                repetition.difficulty as "repetition_difficulty: _",
                repetition.elapsed_days as "repetition_elapsed_days: _",
                repetition.scheduled_days as "repetition_scheduled_days",
                repetition.reps as "repetition_reps: _",
                repetition.lapses as "repetition_lapses: _",
                repetition.state as "repetition_state: _",
                repetition.last_review as "repetition_last_review: _",
                repetition.additional_content as "repetition_additional_content: _"

            FROM cells As cell
            LEFT JOIN repetitions AS repetition ON repetition.cell_id = cell.id

            WHERE searchable_content LIKE $1
            LIMIT 150"#,
            pattern
        )
        .fetch_all(&*self.pool)
        .await;

        match rows {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(rows) => {
                let cells = convert_rows_to_cells(rows);
                Ok(cells)
            },
        }
    }
}


#[cfg(test)]
pub mod tests {
    use crate::{
        cells::entities::cell::CellType, common::{
            sqlite_repositories_context::SqliteRepositoriesContext,
            traits::repositories_context::RepositoriesContext,
        }, file_system::entities::file::File, ROOT_FOLDER_ID
    };

    use super::*;

    #[tokio::test]
    pub async fn get_file_cells_ordered_by_index() {
        // Arrange

        let mut context = SqliteRepositoriesContext::create_testing_context().await;

        let file = File::new_unchecked(None, Some(ROOT_FOLDER_ID), "test".try_into().unwrap());
        context.file_repository().create(&file).await.unwrap();

        let cells = [
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 0),
            Cell::new(None, file.id(), "".to_string(), CellType::Note, 1),
        ];

        context.cell_repository().create(&cells[1]).await.unwrap();
        context.cell_repository().create(&cells[0]).await.unwrap();

        context.save_changes().await.unwrap();

        // Act

        let actual = context
            .cell_repository()
            .get_file_cells_ordered_by_index(file.id())
            .await
            .unwrap();

        // Assert

        assert_eq!(actual[0].id(), cells[0].id());
        assert_eq!(actual[1].id(), cells[1].id());
    }

    #[tokio::test]
    pub async fn search_cells_valid_input_searched_cells_correctly() {
        // Arrange

        let mut context = SqliteRepositoriesContext::create_testing_context().await;

        let file = File::new_unchecked(None, Some(ROOT_FOLDER_ID), "test".try_into().unwrap());
        context.file_repository().create(&file).await.unwrap();

        let cells = [
            Cell::new(None, file.id(), "Test 1".to_string(), CellType::Note, 0),
            Cell::new(None, file.id(), "Test 2".to_string(), CellType::Note, 1),
            Cell::new(None, file.id(), "Not include".to_string(), CellType::Note, 1),
        ];

        context.cell_repository().create(&cells[1]).await.unwrap();
        context.cell_repository().create(&cells[0]).await.unwrap();

        context.save_changes().await.unwrap();

        // Act

        let actual = context
            .cell_repository()
            .search_cells("test")
            .await
            .unwrap();

        // Assert

        assert_eq!(2, actual.len());
        assert!(actual.iter().any(|cell| cell.id() == cells[0].id()));
        assert!(actual.iter().any(|cell| cell.id() == cells[1].id()));
    }
}
