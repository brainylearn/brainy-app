use std::{str::FromStr, sync::Arc};

use sqlx::{
    Sqlite, SqlitePool, Transaction,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use thiserror::Error;
use tokio::sync::Mutex;

use crate::common::traits::repositories_context::RepositoriesContextError;

pub struct SqliteRepositoriesContext {
    pub pool: Arc<SqlitePool>,
    tx: Arc<Mutex<Transaction<'static, Sqlite>>>,
}

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum SqliteRepositoriesContextError {
    #[error("{0}")]
    RepositoriesContextError(#[from] RepositoriesContextError),
    #[error("Sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Migration error")]
    MigrationError(#[from] sqlx::migrate::MigrateError),
}

impl SqliteRepositoriesContext {
    /// Creates a new instance with the url provided, be aware the the migrations
    /// are automatically applied!
    pub async fn new_with_migration(path: &str) -> Result<Self, SqliteRepositoriesContextError> {
        let url = format!("{path}?mode=rwc");
        let options = SqliteConnectOptions::from_str(&url)?
            // Since there is a single client, we can allow read uncommitted, and use shared cache.
            .shared_cache(true)
            .pragma("read_uncommitted", "TRUE")
            .optimize_on_close(true, None);
        let pool = SqlitePoolOptions::new().connect_with(options).await?;
        sqlx::migrate!("./migrations/").run(&pool).await?;

        let arc_pool = Arc::new(pool);
        let tx = Arc::new(Mutex::new(create_transaction(&arc_pool).await));

        Ok(Self {
            pool: arc_pool.clone(),
            tx: tx.clone(),
        })
    }

    /// Returns the old transaction.
    async fn replace_current_transaction_with_new_one(&self) -> Transaction<'static, Sqlite> {
        let mut guard = self.tx.lock().await;
        let new_tx = create_transaction(&self.pool).await;
        std::mem::replace(&mut *guard, new_tx)
    }

    #[cfg(any(test, feature = "test-utils"))]
    /// Creates an in-memory context with migration for testing.
    pub async fn create_testing_context() -> Self {
        SqliteRepositoriesContext::new_with_migration("sqlite::memory:")
            .await
            .unwrap()
    }
}

async fn create_transaction(pool: &Arc<SqlitePool>) -> Transaction<'static, Sqlite> {
    #[cfg(debug_assertions)]
    log::info!("Starting new transaction");
    pool.begin().await.expect("Cannot create a new transaction")
}
