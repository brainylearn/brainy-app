use std::{
    collections::VecDeque,
    sync::Arc,
};

use async_trait::async_trait;
use brainy_core::{
    common::repository_error::RepositoryError,
    file_system::{folder::Folder, folder_repository::FolderRepository, path::Path},
};
use sqlx::{Sqlite, Transaction, sqlite::SqlitePool};
use tokio::sync::Mutex;

// TODO: move
#[derive(sqlx::FromRow)]
struct FolderRow {
    id: uuid::fmt::Hyphenated,
    path: String,
}

impl From<FolderRow> for Folder {
    fn from(value: FolderRow) -> Self {
        Folder::new(Some(value.id.into()), Path::new(&value.path))
    }
}

impl From<&FolderRow> for Folder {
    fn from(value: &FolderRow) -> Self {
        Folder::new(Some(value.id.into()), Path::new(&value.path))
    }
}

pub struct SqliteFolderRepository {
    pub pool: Arc<SqlitePool>,
    pub tx: Arc<Mutex<Option<Transaction<'static, Sqlite>>>>,
}

// TODO: use query! macro
#[async_trait]
impl FolderRepository for SqliteFolderRepository {
    async fn get_all_files(&self) -> Result<Vec<Folder>, RepositoryError> {
        let rows = sqlx::query_as::<_, FolderRow>(
            "SELECT * FROM folders",
        )
        .fetch_all(&*self.pool)
        .await;

        match rows {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(rows) => Ok(rows.into_iter().map(|row| row.into()).collect()),
        }
    }

    async fn get_by_path(&self, path: &Path) -> Result<Option<Folder>, RepositoryError> {
        let row = sqlx::query_as::<_, FolderRow>(
            "SELECT * FROM folders WHERE path = $1",
        )
        .bind(path.to_string())
        .fetch_optional(&*self.pool)
        .await;

        if let Err(err) = row {
            return Err(RepositoryError::UnknownError(err.to_string()));
        }

        let row = row.unwrap();

        match row {
            Some(row) => Ok(Some(row.into())),
            None => Ok(None),
        }
    }

    async fn folder_exists(&self, path: &Path) -> Result<bool, RepositoryError> {
        let row =
            sqlx::query_as::<_, (bool,)>("SELECT EXISTS (SELECT * FROM folders WHERE path = $1)")
                .bind(path.to_string())
                .fetch_one(&*self.pool)
                .await;

        match row {
            Ok(row) => Ok(row.0),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }
}
