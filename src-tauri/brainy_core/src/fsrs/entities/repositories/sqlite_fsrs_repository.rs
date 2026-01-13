use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    Guid,
    common::repository_error::RepositoryError,
    fsrs::entities::{
        fsrs_profile::FsrsProfile,
        repositories::{
            sqlite_fsrs_repository::fsrs_profile_row::FsrsProfileRow,
            traits::fsrs_repository::FsrsRepository,
        },
    },
};

pub struct SqliteFsrsRepository {
    pool: Arc<SqlitePool>,
    tx: Arc<Mutex<Transaction<'static, Sqlite>>>,
}

impl SqliteFsrsRepository {
    pub fn new(pool: Arc<SqlitePool>, tx: Arc<Mutex<Transaction<'static, Sqlite>>>) -> Self {
        Self { pool, tx }
    }
}

// TODO: fsrs profile should not be deletable if there is a folder or a file referencing it
#[async_trait]
impl FsrsRepository for SqliteFsrsRepository {
    // TODO: unit test
    async fn get_by_id(&self, id: Guid) -> Result<FsrsProfile, RepositoryError> {
        let row = sqlx::query_as!(
            FsrsProfileRow,
            r#"SELECT
                id as "id: _",
                name,
                request_retention as "request_retention: _",
                maximum_interval as "maximum_interval: _",
                weights
            FROM fsrs_profiles
            WHERE id = $1"#,
            id
        )
        .fetch_one(&*self.pool)
        .await;

        match row {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(row) => Ok(row.into()),
        }
    }

    // TODO: unit test
    async fn get_all_fsrs_profiles(&self) -> Result<Vec<FsrsProfile>, RepositoryError> {
        let rows = sqlx::query_as!(
            FsrsProfileRow,
            r#"SELECT
                id as "id: _",
                name,
                request_retention as "request_retention: _",
                maximum_interval as "maximum_interval: _",
                weights
            FROM fsrs_profiles"#,
        )
        .fetch_all(&*self.pool)
        .await;

        match rows {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(rows) => Ok(rows.into_iter().map(|row| row.into()).collect()),
        }
    }

    // TODO: unit test
    async fn create(&self, fsrs_profile: &FsrsProfile) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let id = fsrs_profile.id();
        let name = fsrs_profile.name();
        let request_retention = fsrs_profile.request_retention();
        let maximum_interval = fsrs_profile.maximum_interval();
        let weights = fsrs_profile
            .weights()
            .iter()
            .map(|val| val.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        let result = sqlx::query!(
            "INSERT INTO fsrs_profiles(
                id,
                name,
                request_retention,
                maximum_interval,
                weights)
            VALUES ($1, $2, $3, $4, $5)",
            id,
            name,
            request_retention,
            maximum_interval,
            weights
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    // TODO: unit test
    async fn update(&self, fsrs_profile: &FsrsProfile) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let id = fsrs_profile.id();
        let name = fsrs_profile.name();
        let request_retention = fsrs_profile.request_retention();
        let maximum_interval = fsrs_profile.maximum_interval();
        let weights = fsrs_profile
            .weights()
            .iter()
            .map(|val| val.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        let result = sqlx::query!(
            "UPDATE fsrs_profiles SET
                id = $1,
                name = $2,
                request_retention = $3,
                maximum_interval = $4,
                weights = $5
            WHERE id = $1",
            id,
            name,
            request_retention,
            maximum_interval,
            weights
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }
}

mod fsrs_profile_row {
    use crate::Guid;

    use super::*;

    pub(super) struct FsrsProfileRow {
        pub id: Guid,
        pub name: String,
        pub request_retention: f64,
        pub maximum_interval: f64,
        pub weights: String,
    }

    impl From<FsrsProfileRow> for FsrsProfile {
        fn from(value: FsrsProfileRow) -> Self {
            let weights = value
                .weights
                .split(' ')
                .map(|v| v.parse().unwrap())
                .collect();
            FsrsProfile::new(
                Some(value.id),
                value.name,
                value.request_retention,
                value.maximum_interval,
                weights,
            )
        }
    }
}
