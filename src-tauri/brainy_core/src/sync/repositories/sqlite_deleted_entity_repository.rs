use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, Transaction};
use tokio::sync::Mutex;

use crate::{
    Guid, common::repository_error::RepositoryError,
    sync::repositories::traits::DeletedEntityRepository,
};

pub struct SqliteDeletedEntityRepository {
    tx: Arc<Mutex<Transaction<'static, Sqlite>>>,
}

impl SqliteDeletedEntityRepository {
    pub fn new(tx: Arc<Mutex<Transaction<'static, Sqlite>>>) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl DeletedEntityRepository for SqliteDeletedEntityRepository {
    async fn apply_deleted_entity(
        &self,
        entity_name: &str,
        entity_id: Guid,
        delete_date: DateTime<Utc>,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let result = sqlx::query(&format!("DELETE FROM {entity_name} WHERE id = $1"))
            .bind(entity_id)
            .execute(&mut *tx)
            .await;

        if let Err(err) = result {
            return Err(RepositoryError::UnknownError(err.to_string()));
        }

        let result = sqlx::query!(
            r#"UPDATE deleted_entities
                SET delete_date = $1
                WHERE entity_name = $2 AND entity_id = $3
            "#,
            delete_date,
            entity_name,
            entity_id
        )
        .execute(&mut *tx)
        .await;

        if let Err(err) = result {
            return Err(RepositoryError::UnknownError(err.to_string()));
        }

        Ok(())
    }
}
