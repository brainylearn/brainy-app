use std::sync::Arc;

use crate::file_system::repositories::{
    file_repository::FileRepository, folder_repository::FolderRepository,
};
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

pub struct RepositoriesContext {
    pool: Arc<SqlitePool>,
    tx: Arc<Mutex<Option<Transaction<'static, Sqlite>>>>,
    folder_repository: Arc<FolderRepository>,
    file_repository: Arc<FileRepository>,
}

impl RepositoriesContext {
    pub fn new(pool: SqlitePool) -> Self {
        let arc_pool = Arc::new(pool);
        let tx = Arc::new(Mutex::new(None));
        Self {
            pool: arc_pool.clone(),
            tx: tx.clone(),
            file_repository: Arc::new(FileRepository::new(arc_pool.clone(), tx.clone())),
            folder_repository: Arc::new(FolderRepository::new(arc_pool.clone(), tx.clone())),
        }
    }

    pub fn folder_repository(&self) -> Arc<FolderRepository> {
        self.folder_repository.clone()
    }

    pub fn file_repository(&self) -> Arc<FileRepository> {
        self.file_repository.clone()
    }

    pub async fn start(&mut self) {
        if let Some(tx) = self.tx.lock().await.take() {
            // TODO: error handling
            tx.rollback().await.unwrap();
        }

        *self.tx.lock().await = Some(self.pool.begin().await.unwrap());
    }

    pub async fn commit(&mut self) {
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
