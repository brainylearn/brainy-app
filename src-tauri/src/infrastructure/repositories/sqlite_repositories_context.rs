use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{domain::repositories::{folder_repository::FolderRepository, repositories_context::RepositoriesContext}, infrastructure::repositories::sqlite_folder_repository::SqliteFolderRepository};

pub struct SqliteRepositoriesContext {
    pool: Arc<SqlitePool>,
    folder_repository: SqliteFolderRepository,
    tx: Arc<Mutex<Option<Transaction<'static, Sqlite>>>>,
}

impl SqliteRepositoriesContext {
    pub fn new(pool: SqlitePool) -> Self {
        let arc_pool = Arc::new(pool);
        let tx = Arc::new(Mutex::new(None));
        Self {
            pool: arc_pool.clone(),
            folder_repository: SqliteFolderRepository { pool: arc_pool.clone(), tx: tx.clone() },
            tx,
        }
    }
}

#[async_trait]
impl RepositoriesContext for SqliteRepositoriesContext {
    fn folder_repository(&mut self) -> Box<&mut dyn FolderRepository> {
        Box::new(&mut self.folder_repository)
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
