use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    Guid,
    cells::{
        entities::cell::{Cell, CellType},
        repositories::{
            sqlite_cell_repository::cell_row::CellRow,
            traits::cell_repository::{CellRepository, MoveDirection},
        },
    },
    common::repository_error::RepositoryError,
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
        let row = sqlx::query_as!(
            CellRow,
            r#"SELECT
                id as "id: _",
                file_id as "file_id: _",
                content,
                cell_index as "index: _",
                cell_type as "cell_type: _"
            FROM cells
            WHERE id = $1"#,
            id
        )
        .fetch_one(&*self.pool)
        .await;

        match row {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(row) => Ok(row.into()),
        }
    }

    async fn get_file_cells_ordered_by_index(
        &self,
        file_id: Guid,
    ) -> Result<Vec<Cell>, RepositoryError> {
        let rows = sqlx::query_as!(
            CellRow,
            r#"SELECT
                id as "id: _",
                file_id as "file_id: _",
                content,
                cell_index as "index: _",
                cell_type as "cell_type: _"
            FROM cells
            WHERE file_id = $1
            ORDER BY cell_index"#,
            file_id
        )
        .fetch_all(&*self.pool)
        .await;

        match rows {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(rows) => Ok(rows.into_iter().map(|row| row.into()).collect()),
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

        let result = sqlx::query!(
            r#"INSERT INTO
                cells(id, content, cell_type, cell_index, file_id)
                VALUES ($1, $2, $3, $4, $5)"#,
            id,
            content,
            cell_type,
            index,
            file_id
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

    async fn delete_by_id(&self, id: Guid) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let result = sqlx::query!(r#"DELETE FROM cells WHERE id = $1"#, id,)
            .execute(&mut *tx)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }
}

mod cell_type_sqlite_impls {
    use super::*;

    impl sqlx::Type<Sqlite> for CellType {
        fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
            <str as sqlx::Type<sqlx::Sqlite>>::type_info()
        }
    }

    impl<'r> sqlx::Decode<'r, Sqlite> for CellType {
        fn decode(
            value: <Sqlite as sqlx::Database>::ValueRef<'r>,
        ) -> Result<Self, sqlx::error::BoxDynError> {
            let value = <&'r str as sqlx::decode::Decode<'r, sqlx::sqlite::Sqlite>>::decode(value)?;
            match serde_json::from_str(value) {
                Ok(cell_type) => Ok(cell_type),
                _ => Err(format!("invalid value {:?} for enum {}", value, "CellType").into()),
            }
        }
    }

    impl<'q> sqlx::Encode<'q, Sqlite> for CellType {
        fn encode_by_ref(
            &self,
            buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
        ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
            let val = serde_json::to_string(&self).expect("Cannot serialize CellType");
            <String as sqlx::encode::Encode<'q, Sqlite>>::encode(val, buf)
        }
    }
}

mod cell_row {
    use super::*;

    #[derive(sqlx::FromRow)]
    pub(super) struct CellRow {
        pub id: Guid,
        pub file_id: Guid,
        pub content: String,
        pub cell_type: CellType,
        pub index: u32,
    }

    impl From<CellRow> for Cell {
        fn from(value: CellRow) -> Self {
            Cell::new(
                Some(value.id),
                value.file_id,
                value.content,
                value.cell_type,
                value.index,
            )
        }
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
}
