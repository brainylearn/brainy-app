use std::sync::Arc;

use async_trait::async_trait;
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
    ) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        // TODO: move logging to sync service, also add logging for other entities too
        log::info!("Deleting entity with entity name {entity_name} and id {entity_id}.");

        let result = sqlx::query(&format!("DELETE FROM {entity_name} WHERE id = $1"))
            .bind(entity_id)
            .execute(&mut *tx)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }
}
