use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use sqlx::{
    Sqlite, SqlitePool, Transaction,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use thiserror::Error;
use tokio::sync::Mutex;

use crate::{
    common::traits::repositories_context::{RepositoriesContext, RepositoriesContextError},
    file_system::repositories::{
        sqlite_file_repository::SqliteFileRepository,
        sqlite_folder_repository::SqliteFolderRepository,
        traits::{file_repository::FileRepository, folder_repository::FolderRepository},
    },
};

pub struct SqliteRepositoriesContext {
    pool: Arc<SqlitePool>,
    tx: Arc<Mutex<Transaction<'static, Sqlite>>>,
    folder_repository: Arc<SqliteFolderRepository>,
    file_repository: Arc<SqliteFileRepository>,
}

#[derive(Debug, Error)]
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
    pub async fn new_with_migration(url: &str) -> Result<Self, SqliteRepositoriesContextError> {
        let options = SqliteConnectOptions::from_str(url)?;
        let pool = SqlitePoolOptions::new().connect_with(options).await?;
        sqlx::migrate!("./db/").run(&pool).await?;

        let arc_pool = Arc::new(pool);
        let tx = Arc::new(Mutex::new(create_transactoin(arc_pool.clone()).await));

        Ok(Self {
            pool: arc_pool.clone(),
            tx: tx.clone(),
            file_repository: Arc::new(SqliteFileRepository::new(arc_pool.clone(), tx.clone())),
            folder_repository: Arc::new(SqliteFolderRepository::new(arc_pool.clone(), tx.clone())),
        })
    }

    #[cfg(test)]
    pub async fn create_in_memory_context() -> Self {
        SqliteRepositoriesContext::new_with_migration("sqlite::memory:")
            .await
            .unwrap()
    }
}

#[async_trait]
impl RepositoriesContext for SqliteRepositoriesContext {
    fn folder_repository(&self) -> Arc<dyn FolderRepository> {
        self.folder_repository.clone()
    }

    fn file_repository(&self) -> Arc<dyn FileRepository> {
        self.file_repository.clone()
    }

    async fn save_changes(&mut self) -> Result<(), RepositoriesContextError> {
        log::info!("Saving changes");
        let mut guard = self.tx.lock().await;

        let new_tx = create_transactoin(self.pool.clone()).await;
        let old_tx = std::mem::replace(&mut *guard, new_tx);

        if let Err(err) = old_tx.commit().await {
            return Err(RepositoriesContextError::UnknownError(err.to_string()));
        }
        Ok(())
    }
}

async fn create_transactoin(pool: Arc<SqlitePool>) -> Transaction<'static, Sqlite> {
    log::info!("Starting new transaction");
    pool.begin().await.expect("Cannot create a new transaction")
}
