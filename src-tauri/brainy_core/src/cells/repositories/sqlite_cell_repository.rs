use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    Guid,
    cells::{
        entities::cell::{Cell, CellType},
        repositories::traits::cell_repository::CellRepository,
    },
    common::repository_error::RepositoryError,
};

// TODO: move, do the same for folder and files
impl sqlx::Type<Sqlite> for CellType {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <str as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, Sqlite> for CellType {
    fn decode(value: <Sqlite as sqlx::Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <&'r str as sqlx::decode::Decode<'r, sqlx::sqlite::Sqlite>>::decode(value)?;
        match serde_json::from_str(value) {
            Ok(cell_type) => Ok(cell_type),
            _ => Err(format!("invalid value {:?} for enum {}", value, "CellType").into()),
        }
    }
}

impl <'q> sqlx::Encode<'q, Sqlite> for CellType {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let val = serde_json::to_string(&self).expect("Cannot serialize CellType");
        <String as sqlx::encode::Encode<'q, Sqlite>>::encode(val, buf)
    }
}

#[derive(sqlx::FromRow)]
struct CellRow {
    id: Guid,
    file_id: Guid,
    content: String,
    cell_type: CellType,
    index: u32,
}

impl From<CellRow> for Cell {
    fn from(value: CellRow) -> Self {
        Cell::new(
            value.id,
            value.file_id,
            value.content,
            value.cell_type,
            value.index,
        )
    }
}

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
    // TODO: unit test
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
}
