use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use sqlx::{
    Sqlite, SqlitePool, Transaction,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
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
    tx: Arc<Mutex<Option<Transaction<'static, Sqlite>>>>,
    folder_repository: Arc<SqliteFolderRepository>,
    file_repository: Arc<SqliteFileRepository>,
}

impl SqliteRepositoriesContext {
    /// Creates a new instance with the url provided, be aware the the migrations
    /// are automatically applied!
    pub async fn new_with_migration(url: &str) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::from_str(url)?;
        let pool = SqlitePoolOptions::new().connect_with(options).await?;
        sqlx::migrate!("./db/").run(&pool).await?;

        let arc_pool = Arc::new(pool);
        let tx = Arc::new(Mutex::new(None));
        Ok(Self {
            pool: arc_pool.clone(),
            tx: tx.clone(),
            file_repository: Arc::new(SqliteFileRepository::new(arc_pool.clone(), tx.clone())),
            folder_repository: Arc::new(SqliteFolderRepository::new(arc_pool.clone(), tx.clone())),
        })
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

    async fn start(&mut self) -> Result<(), RepositoriesContextError> {
        if let Some(tx) = self.tx.lock().await.take() {
            match tx.rollback().await {
                Err(err) => return Err(RepositoriesContextError::UnknownError(err.to_string())),
                _ => (),
            };
        }

        log::info!("Starting new transaction");
        match self.pool.begin().await {
            Err(err) => Err(RepositoriesContextError::UnknownError(err.to_string())),
            Ok(val) => {
                *self.tx.lock().await = Some(val);
                return Ok(());
            }
        }
    }

    async fn commit(&mut self) -> Result<(), RepositoriesContextError> {
        log::info!("Commiting");
        if let Some(tx) = self.tx.lock().await.take() {
            log::info!("Saving changes");

            if let Err(err) = tx.commit().await {
                return Err(RepositoriesContextError::UnknownError(err.to_string()));
            }
        } else {
            return Err(RepositoriesContextError::TransactionNotStarted);
        }

        *self.tx.lock().await = None;
        Ok(())
    }
}
