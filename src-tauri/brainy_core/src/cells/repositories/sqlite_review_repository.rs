use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    cells::{
        entities::review::Review,
        repositories::{
            sqlite_review_repository::review_row::ReviewRow,
            traits::review_repository::ReviewRepository,
        },
    },
    common::repository_error::RepositoryError,
};

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
impl ReviewRepository for SqliteReviewRepository {
    async fn create(&self, review: &Review) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let Review {
            id,
            created_date,
            cell_id,
            study_time,
            date,
            rating,
        } = review;

        let result = sqlx::query!(
            r#"INSERT INTO
                reviews(id, created_date, cell_id, study_time, date, rating)
                VALUES ($1, $2, $3, $4, $5, $6)"#,
            id,
            created_date,
            cell_id,
            study_time,
            date,
            rating
        )
        .execute(&mut *tx)
        .await;

        match result {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(_) => Ok(()),
        }
    }

    async fn get_all_modified_on_or_after(
        &self,
        modified_date: DateTime<Utc>,
    ) -> Result<Vec<Review>, RepositoryError> {
        let rows = sqlx::query_as!(
            ReviewRow,
            r#"SELECT
                id as "id: _",
                created_date as "created_date: _",
                cell_id as "cell_id: _",
                study_time as "study_time: _",
                date as "date: _",
                rating as "rating: _"
            FROM reviews
            WHERE modified_date >= datetime($1)"#,
            modified_date
        )
        .fetch_all(&*self.pool)
        .await;

        match rows {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(rows) => Ok(rows.into_iter().map(|row| row.into()).collect()),
        }
    }
}

mod review_row {
    use chrono::{DateTime, Utc};

    use crate::{Guid, cells::entities::review::Rating};

    use super::*;

    pub(super) struct ReviewRow {
        pub id: Guid,
        pub created_date: DateTime<Utc>,
        pub cell_id: Option<Guid>,
        pub study_time: u32,
        pub date: DateTime<Utc>,
        pub rating: Rating,
    }

    impl From<ReviewRow> for Review {
        fn from(value: ReviewRow) -> Self {
            Review::new_unchecked(
                value.id,
                value.created_date,
                value.cell_id,
                value.study_time,
                value.date,
                value.rating,
            )
        }
    }
}
