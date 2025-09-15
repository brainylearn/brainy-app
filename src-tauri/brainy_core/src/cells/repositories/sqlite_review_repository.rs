use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::cells::repositories::traits::review_repository::ReviewRepository;

pub struct SqliteReviewRepository {
    pool: Arc<SqlitePool>,
    tx: Arc<Mutex<Transaction<'static, Sqlite>>>,
}

impl SqliteReviewRepository {
    pub fn new(pool: Arc<SqlitePool>, tx: Arc<Mutex<Transaction<'static, Sqlite>>>) -> Self {
        Self { pool, tx }
    }
}

#[async_trait]
impl ReviewRepository for SqliteReviewRepository {}
