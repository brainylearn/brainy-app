use std::sync::Arc;

use async_trait::async_trait;
use injector_derive::ScopeInjectable;

use crate::{
    Guid,
    common::repository_error::RepositoryError,
    incremental_reading::extracts::{
        entities::extract::Extract, repositories::extract_repository::ExtractRepository,
    },
    infrastructure::{
        repositories::sqlite::sqlite_rows::extract_row::ExtractRow,
        value_objects::db_transaction::DbTransaction,
    },
};

#[derive(ScopeInjectable)]
pub struct SqliteExtractRepository {
    tx: Arc<DbTransaction>,
}

#[async_trait]
impl ExtractRepository for SqliteExtractRepository {
    async fn get_by_cell_id(&self, cell_id: Guid) -> Result<Vec<Extract>, RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let rows = sqlx::query_as!(
            ExtractRow,
            r#"SELECT
                id as "id: _",
                created_date as "created_date: _",
                modified_date as "modified_date: _",
                cell_id as "cell_id: _",
                status as "status: _"
            FROM extracts
            WHERE cell_id = $1"#,
            cell_id
        )
        .fetch_all(&mut *tx)
        .await?;

        Ok(rows.into_iter().map(Extract::from).collect())
    }

    async fn create(&self, extract: &Extract) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        sqlx::query!(
            r#"INSERT INTO extracts(
                id,
                created_date,
                modified_date,
                cell_id,
                status)
            VALUES ($1, datetime($2), datetime($3), $4, $5)"#,
            extract.id(),
            extract.created_date(),
            extract.modified_date(),
            extract.cell_id(),
            extract.status(),
        )
        .execute(&mut *tx)
        .await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Guid) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        sqlx::query!("DELETE FROM extracts WHERE id = $1", id)
            .execute(&mut *tx)
            .await?;

        Ok(())
    }
}
