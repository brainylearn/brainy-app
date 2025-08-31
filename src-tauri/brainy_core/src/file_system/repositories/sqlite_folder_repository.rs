use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    Guid,
    common::repository_error::RepositoryError,
    file_system::{
        entities::folder::Folder, repositories::traits::folder_repository::FolderRepository,
        value_objects::file_system_item_name::FileSystemItemName,
    },
};

#[derive(sqlx::FromRow)]
struct FolderRow {
    id: Guid,
    parent_id: Option<Guid>,
    name: String,
}

impl From<FolderRow> for Folder {
    fn from(value: FolderRow) -> Self {
        Folder::new(
            Some(value.id),
            value.parent_id,
            FileSystemItemName::new_unchecked(value.name),
        )
    }
}

pub struct SqliteFolderRepository {
    pool: Arc<SqlitePool>,
    tx: Arc<Mutex<Transaction<'static, Sqlite>>>,
}

impl SqliteFolderRepository {
    pub fn new(pool: Arc<SqlitePool>, tx: Arc<Mutex<Transaction<'static, Sqlite>>>) -> Self {
        Self { pool, tx }
    }
}

#[async_trait]
impl FolderRepository for SqliteFolderRepository {
    async fn get_by_id(&self, id: Guid) -> Result<Folder, RepositoryError> {
        let row = sqlx::query_as!(
            FolderRow,
            r#"SELECT id as "id: _", parent_id as "parent_id: _", name FROM folders WHERE id = $1"#,
            id
        )
        .fetch_one(&*self.pool)
        .await;

        match row {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(row) => Ok(row.into()),
        }
    }

    async fn get_all_folders(&self) -> Result<Vec<Folder>, RepositoryError> {
        let rows = sqlx::query_as!(
            FolderRow,
            r#"SELECT id as "id: _", parent_id as "parent_id: _", name FROM folders"#,
        )
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
        let name_string = name.to_string();
        let row = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM folders WHERE parent_id = $1 AND name = $2"#,
            parent_id,
            name_string
        )
        .fetch_one(&*self.pool)
        .await;

        match row {
            Ok(cnt) => Ok(cnt > 0),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn create(&self, folder: &Folder) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let folder_id = folder.id();
        let folder_name = folder.name().to_string();
        let parent_id = folder.parent_id();
        let result = sqlx::query!(
            "INSERT INTO folders(id, name, parent_id) VALUES ($1, $2, $3)",
            folder_id,
            folder_name,
            parent_id
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn update(&self, folder: &Folder) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let folder_id = folder.id();
        let folder_name = folder.name().to_string();
        let parent_id = folder.parent_id();
        let result = sqlx::query!(
            "UPDATE folders SET id = $1, name = $2, parent_id = $3 WHERE id = $1",
            folder_id,
            folder_name,
            parent_id
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn delete_by_id(&self, id: Guid) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let result = sqlx::query!("DELETE FROM folders WHERE id = $1", id)
            .execute(&mut *tx)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }
}
