use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::{domain::repositories::{folder_repository::FolderRepository, repositories_context::RepositoriesContext}, infrastructure::repositories::sqlite_folder_repository::SqliteFolderRepository};

pub struct SqliteRepositoriesContext {
    pool: Arc<SqlitePool>,
    folder_repository: SqliteFolderRepository,
    tx: Arc<Option<Transaction<'static, Sqlite>>>,
}

impl SqliteRepositoriesContext {
    pub fn new(pool: SqlitePool) -> Self {
        let arc_pool = Arc::new(pool);
        Self {
            pool: arc_pool.clone(),
            folder_repository: SqliteFolderRepository { pool: arc_pool.clone(), tx: Arc::new(None) },
            tx: Arc::new(None),
        }
    }
}

#[async_trait]
impl RepositoriesContext for SqliteRepositoriesContext {
    fn folder_repository(&self) -> Box<&dyn FolderRepository> {
        Box::new(&self.folder_repository)
    }

    async fn start(&mut self) {
        // TODO: error handling
        if let Some(tx) = Arc::get_mut(&mut self.tx).unwrap().take() {
            tx.rollback().await.unwrap();
        }

        let tx = Arc::new(Some(self.pool.begin().await.unwrap()));
        // NOTE: update all repsitories
        self.folder_repository.tx = tx.clone();
        self.tx = tx.clone();
    }

    async fn commit(&mut self) {
        if let Some(tx) = Arc::get_mut(&mut self.tx).unwrap().take() {
            tx.commit().await.unwrap();
        } 
        // NOTE: update all repsitories
        self.folder_repository.tx = Arc::new(None);

        // TODO: throw error when not started
    }
}
