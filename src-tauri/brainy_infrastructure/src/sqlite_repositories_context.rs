use std::sync::Arc;

use async_trait::async_trait;
use brainy_core::file_system::repositories::{
    file_repository::FileRepository, folder_repository::FolderRepository,
};
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    file_system::repositories::{
        sqlite_file_repository::SqliteFileRepository,
        sqlite_folder_repository::SqliteFolderRepository,
    },
    repositories_context::RepositoriesContext,
};

pub struct SqliteRepositoriesContext {
    pool: Arc<SqlitePool>,
    tx: Arc<Mutex<Option<Transaction<'static, Sqlite>>>>,
}

impl SqliteRepositoriesContext {
    pub fn new(pool: SqlitePool) -> Self {
        let arc_pool = Arc::new(pool);
        let tx = Arc::new(Mutex::new(None));
        Self {
            pool: arc_pool.clone(),
            tx,
        }
    }
}

#[async_trait]
impl RepositoriesContext for SqliteRepositoriesContext {
    fn folder_repository(&self) -> Box<dyn FolderRepository> {
        Box::new(SqliteFolderRepository {
            pool: self.pool.clone(),
            tx: self.tx.clone(),
        })
    }

    fn file_repository(&self) -> Box<dyn FileRepository> {
        Box::new(SqliteFileRepository {
            pool: self.pool.clone(),
            tx: self.tx.clone(),
        })
    }

    async fn start(&mut self) {
        if let Some(tx) = self.tx.lock().await.take() {
            // TODO: error handling
            tx.rollback().await.unwrap();
        }

        *self.tx.lock().await = Some(self.pool.begin().await.unwrap());
    }

    async fn commit(&mut self) {
        println!("Commiting");
        if let Some(tx) = self.tx.lock().await.take() {
            // TODO: error handling
            println!("Saving changes");
            tx.commit().await.unwrap();
        }

        *self.tx.lock().await = None;

        // TODO: throw error when not started
    }
}
