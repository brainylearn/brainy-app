use std::sync::Arc;

use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::common::traits::repositories_context::RepositoriesContextError;

pub struct UnitOfWork {
    pool: Arc<SqlitePool>,
    // TODO: use alias
    tx: Arc<Mutex<Transaction<'static, Sqlite>>>,
}

impl UnitOfWork {
    pub fn new(pool: Arc<SqlitePool>, tx: Arc<Mutex<Transaction<'static, Sqlite>>>) -> Self {
        Self { pool, tx }
    }

    pub async fn save_changes(&self) -> Result<(), RepositoriesContextError> {
        log::info!("Saving changes");

        let old_tx = self.replace_current_transaction_with_new_one().await;

        if let Err(err) = old_tx.commit().await {
            return Err(RepositoriesContextError::UnknownError(err.to_string()));
        }
        Ok(())
    }

    pub async fn rollback(&self) -> Result<(), RepositoriesContextError> {
        log::info!("Aborting transaction");

        let old_tx = self.replace_current_transaction_with_new_one().await;

        if let Err(err) = old_tx.rollback().await {
            return Err(RepositoriesContextError::UnknownError(err.to_string()));
        }
        Ok(())
    }

    pub async fn disable_foreign_key_constraint_for_current_transaction(
        &self,
    ) -> Result<(), RepositoriesContextError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let result = sqlx::query("PRAGMA defer_foreign_keys = ON")
            .fetch_optional(&mut *tx)
            .await;

        if let Err(err) = result {
            return Err(RepositoriesContextError::UnknownError(err.to_string()));
        }

        Ok(())
    }

    /// Returns the old transaction.
    async fn replace_current_transaction_with_new_one(&self) -> Transaction<'static, Sqlite> {
        let mut guard = self.tx.lock().await;
        let new_tx = create_transaction(&self.pool).await;
        std::mem::replace(&mut *guard, new_tx)
    }
}

async fn create_transaction(pool: &Arc<SqlitePool>) -> Transaction<'static, Sqlite> {
    #[cfg(debug_assertions)]
    log::info!("Starting new transaction");
    pool.begin().await.expect("Cannot create a new transaction")
}
