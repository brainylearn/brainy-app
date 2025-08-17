use std::sync::Arc;

use async_trait::async_trait;
use brainy_core::{
    Guid,
    common::repository_error::RepositoryError,
    file_system::{
        entities::folder::Folder, repositories::folder_repository::FolderRepository,
        value_objects::file_system_item_name::FileSystemItemName,
    },
};
use sqlx::{Sqlite, Transaction, sqlite::SqlitePool};
use tokio::sync::Mutex;

#[derive(sqlx::FromRow)]
struct FolderRow {
    id: Guid,
    parent_id: Option<Guid>,
    name: String,
}

impl From<FolderRow> for Folder {
    fn from(value: FolderRow) -> Self {
        Folder::new_unchecked(
            Some(value.id.into()),
            value.parent_id,
            FileSystemItemName::new_unchecked(value.name),
        )
    }
}

impl From<&FolderRow> for Folder {
    fn from(value: &FolderRow) -> Self {
        Folder::new_unchecked(
            Some(value.id.into()),
            value.parent_id,
            FileSystemItemName::new_unchecked(value.name.clone()),
        )
    }
}

pub struct SqliteFolderRepository {
    pub pool: Arc<SqlitePool>,
    pub tx: Arc<Mutex<Option<Transaction<'static, Sqlite>>>>,
}

// TODO: use query! macro
#[async_trait]
impl FolderRepository for SqliteFolderRepository {
    async fn get_all_folders(&self) -> Result<Vec<Folder>, RepositoryError> {
        let rows = sqlx::query_as::<_, FolderRow>("SELECT * FROM folders")
            .fetch_all(&*self.pool)
            .await;

        match rows {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(rows) => Ok(rows.into_iter().map(|row| row.into()).collect()),
        }
    }

    async fn exists(
        &self,
        parent_id: Option<Guid>,
        name: &FileSystemItemName,
    ) -> Result<bool, RepositoryError> {
        let row = sqlx::query_as::<_, (bool,)>(
            "SELECT EXISTS (SELECT * FROM folders WHERE parent_id = $1 AND name = $2)",
        )
        .bind(parent_id)
        .bind(name.to_string())
        .fetch_one(&*self.pool)
        .await;

        match row {
            Ok(row) => Ok(row.0),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn create(&self, folder: &Folder) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut().unwrap();

        let result = sqlx::query("INSERT INTO folders(id, name, parent_id) VALUES ($1, $2, $3)")
            .bind(folder.id())
            .bind(folder.name().to_string())
            .bind(folder.parent_id())
            .execute(&mut *tx.as_mut())
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }
}
