use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{QueryBuilder, Sqlite, SqliteConnection, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    cells::{
        entities::{cell::Cell, repetition::{Repetition, State}},
        repositories::{
            sqlite_rows::cell_row::{convert_rows_to_cells, CellRow, RepetitionRow},
            traits::cell_repository::{CellRepository, MoveDirection},
        },
        value_objects::{cell_deletion_request::CellDeletionRequest, file_repetitions_count::FileRepetitionCounts},
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
            }
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
            }
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

        if let Err(err) = result {
            return Err(RepositoryError::UnknownError(err.to_string()));
        }

        self.upsert_repetitions(tx, cell.repetitions()).await
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

        if let Err(err) = result {
            return Err(RepositoryError::UnknownError(err.to_string()));
        }

        // Deleteing removed repetitions.

        let mut query_builder: QueryBuilder<sqlx::Sqlite> =
            QueryBuilder::new("DELETE FROM repetitions WHERE cell_id = ");
        query_builder.push_bind(id);
        query_builder.push(" AND id NOT IN (");
        let mut separated = query_builder.separated(",");
        for repetition in cell.repetitions() {
            separated.push_bind(repetition.id);
        }
        separated.push_unseparated(")");

        if let Err(err) = query_builder.build().execute(&mut *tx).await {
            return Err(RepositoryError::UnknownError(err.to_string()));
        }

        self.upsert_repetitions(tx, cell.repetitions()).await
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
            }
        }
    }

    async fn get_file_repetitions(
        &self,
        file_id: Guid,
    ) -> Result<Vec<Repetition>, RepositoryError> {
        let rows = sqlx::query_as!(
            RepetitionRow,
            r#"SELECT
                id as "id: _",
                file_id as "file_id: _",
                cell_id as "cell_id: _",
                due as "due: _",
                stability as "stability: _",
                difficulty as "difficulty: _",
                elapsed_days as "elapsed_days: _",
                scheduled_days as "scheduled_days",
                reps as "reps: _",
                lapses as "lapses: _",
                state as "state: _",
                last_review as "last_review: _",
                additional_content as "additional_content: _"

            FROM repetitions
            WHERE file_id = $1"#,
            file_id
        )
        .fetch_all(&*self.pool)
        .await;

        match rows {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(rows) => Ok(rows.into_iter().map(|row| row.into()).collect()),
        }
    }

    // TODO: unit tests
    async fn get_study_repetitions(
        &self,
        file_id: Guid,
    ) -> Result<FileRepetitionCounts, RepositoryError> {
        let now = Utc::now().to_utc();
        let rows = sqlx::query!(
            r#"
                SELECT state AS "state: State", COUNT(*) AS "count: u32"
                FROM repetitions
                WHERE file_id = $1 AND due <= $2
                GROUP BY state
            "#,
            file_id,
            now
        )
        .fetch_all(&*self.pool)
        .await;

        match rows {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(rows) => {
                let mut counts: FileRepetitionCounts = Default::default();

                for row in rows {
                    if row.state == State::New {
                        counts.new = row.count;
                    } else if row.state == State::Learning {
                        counts.learning = row.count;
                    } else if row.state == State::Relearning {
                        counts.relearning = row.count;
                    } else if row.state == State::Review {
                        counts.review = row.count;
                    }
                }

                Ok(counts)
            },
        }
    }
}

impl SqliteCellRepository {
    async fn upsert_repetitions(
        &self,
        tx: &mut SqliteConnection,
        repetitions: &Vec<Repetition>,
    ) -> Result<(), RepositoryError> {
        for repetition in repetitions {
            let Repetition {
                id,
                file_id,
                cell_id,
                due,
                stability,
                difficulty,
                elapsed_days,
                scheduled_days,
                reps,
                lapses,
                state,
                last_review,
                additional_content,
            } = repetition;

            // TODO: maybe INSERT OR REPLACE is too dangerous?
            let result = sqlx::query!(
                r#"INSERT OR REPLACE INTO
                    repetitions(
                        id,
                        file_id,
                        cell_id,
                        due,
                        stability,
                        difficulty,
                        elapsed_days,
                        scheduled_days,
                        reps,
                        lapses,
                        state,
                        last_review,
                        additional_content)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"#,
                id,
                file_id,
                cell_id,
                due,
                stability,
                difficulty,
                elapsed_days,
                scheduled_days,
                reps,
                lapses,
                state,
                last_review,
                additional_content
            )
            .execute(&mut *tx)
            .await;

            if let Err(err) = result {
                return Err(RepositoryError::UnknownError(err.to_string()));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        ROOT_FOLDER_ID,
        cells::entities::cell::CellType,
        common::{
            sqlite_repositories_context::SqliteRepositoriesContext,
            traits::repositories_context::RepositoriesContext,
        },
        file_system::entities::file::File,
    };

    use super::*;

    #[tokio::test]
    pub async fn get_by_id_valid_input_returned_cell_correctly() {
        // Arrange

        let mut context = SqliteRepositoriesContext::create_testing_context().await;

        let file = File::new_unchecked(None, Some(ROOT_FOLDER_ID), "test".try_into().unwrap());
        context.file_repository().create(&file).await.unwrap();

        let cell = Cell::new(
            None,
            file.id(),
            r#"
                <cloze index="1">test<cloze>
                <cloze index="2">test<cloze>
            "#
            .to_string(),
            CellType::Cloze,
            0,
        );
        context.cell_repository().create(&cell).await.unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = context
            .cell_repository()
            .get_by_id(cell.id())
            .await
            .unwrap();

        // Assert

        assert_eq!(cell.id(), actual.id());
        assert_eq!(2, actual.repetitions().len());
        assert!(
            actual
                .repetitions()
                .iter()
                .any(|r| r.additional_content.as_ref().unwrap() == "1")
        );
        assert!(
            actual
                .repetitions()
                .iter()
                .any(|r| r.additional_content.as_ref().unwrap() == "2")
        );
    }

    #[tokio::test]
    pub async fn get_file_cells_ordered_by_index_valid_input_returned_files_ordered() {
        // Arrange

        let mut context = SqliteRepositoriesContext::create_testing_context().await;

        let file = File::new_unchecked(None, Some(ROOT_FOLDER_ID), "test".try_into().unwrap());
        context.file_repository().create(&file).await.unwrap();

        let cells = [
            Cell::new(
                None,
                file.id(),
                r#"<cloze index="1"></cloze>"#.to_string(),
                CellType::Cloze,
                0,
            ),
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

        assert_eq!(cells[0].id(), actual[0].id());
        assert_eq!(1, actual[0].repetitions().len());
        assert_eq!(cells[1].id(), actual[1].id());
    }

    #[tokio::test]
    pub async fn update_deleted_old_repetitions_and_added_new_ones() {
        // Arrange

        let mut context = SqliteRepositoriesContext::create_testing_context().await;

        let file = File::new_unchecked(None, Some(ROOT_FOLDER_ID), "test".try_into().unwrap());
        context.file_repository().create(&file).await.unwrap();

        let mut cell = Cell::new(
            None,
            file.id(),
            r#"
                <cloze index="1">test<cloze>
                <cloze index="2">test<cloze>
            "#
            .to_string(),
            CellType::Cloze,
            0,
        );
        context.cell_repository().create(&cell).await.unwrap();
        context.save_changes().await.unwrap();

        let old_repetitions = cell.repetitions().clone();
        cell.set_content(
            r#"
                <cloze index="1">test<cloze>
                <cloze index="3">test<cloze>
            "#
            .to_string(),
        );

        // Act

        context.cell_repository().update(&cell).await.unwrap();
        context.save_changes().await.unwrap();

        // Assert

        let actual = context
            .cell_repository()
            .get_by_id(cell.id())
            .await
            .unwrap();

        assert_eq!(2, cell.repetitions().len());
        assert!(
            actual
                .repetitions()
                .iter()
                .any(|r| r.additional_content.as_ref().unwrap() == "1"
                    && old_repetitions.iter().any(|r2| r2.id == r.id))
        );
        assert!(
            actual
                .repetitions()
                .iter()
                .any(|r| r.additional_content.as_ref().unwrap() == "3")
        );

        let deleted_repetition_id = old_repetitions
            .iter()
            .find(|r| r.additional_content.as_ref().unwrap() == "2")
            .unwrap()
            .id;
        assert!(
            !cell
                .repetitions()
                .iter()
                .any(|r| r.id == deleted_repetition_id)
        );
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
            Cell::new(
                None,
                file.id(),
                "Not include".to_string(),
                CellType::Note,
                1,
            ),
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

    #[tokio::test]
    pub async fn delete_by_id_cloze_cell_deleted_repetitions() {
        // Arrange

        let mut context = SqliteRepositoriesContext::create_testing_context().await;

        let file = File::new_unchecked(None, Some(ROOT_FOLDER_ID), "test".try_into().unwrap());
        context.file_repository().create(&file).await.unwrap();

        let cell = Cell::new(
            None,
            file.id(),
            r#"
                <cloze index="1">test<cloze>
            "#
            .to_string(),
            CellType::Cloze,
            0,
        );
        context.cell_repository().create(&cell).await.unwrap();
        context.save_changes().await.unwrap();

        // Act

        context
            .cell_repository()
            .delete_by_id(CellDeletionRequest::new(cell.id()))
            .await
            .unwrap();
        context.save_changes().await.unwrap();

        // Assert

        let actual = context
            .cell_repository()
            .get_file_repetitions(file.id())
            .await
            .unwrap();
        assert_eq!(0, actual.len());
    }

    #[tokio::test]
    pub async fn get_file_repetitions_returned_all_repetitions_correctly() {
        // Arrange

        let mut context = SqliteRepositoriesContext::create_testing_context().await;

        let file = File::new_unchecked(None, Some(ROOT_FOLDER_ID), "test".try_into().unwrap());
        context.file_repository().create(&file).await.unwrap();

        let cell = Cell::new(
            None,
            file.id(),
            r#"
                <cloze index="1">test<cloze>
                <cloze index="2">test<cloze>
            "#
            .to_string(),
            CellType::Cloze,
            0,
        );
        context.cell_repository().create(&cell).await.unwrap();
        context.save_changes().await.unwrap();

        // Act

        let actual = context
            .cell_repository()
            .get_file_repetitions(file.id())
            .await
            .unwrap();

        // Assert

        assert_eq!(2, actual.len());
        assert!(
            actual
                .iter()
                .any(|r| r.additional_content.as_ref().unwrap() == "1")
        );
        assert!(
            actual
                .iter()
                .any(|r| r.additional_content.as_ref().unwrap() == "2")
        );
    }
}
