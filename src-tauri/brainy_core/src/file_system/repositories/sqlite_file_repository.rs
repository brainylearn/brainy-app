use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    Guid,
    common::repository_error::RepositoryError,
    file_system::{
        entities::file::File, repositories::traits::file_repository::FileRepository,
        value_objects::file_system_item_name::FileSystemItemName,
    },
};

#[derive(sqlx::FromRow)]
struct FileRow {
    id: Guid,
    parent_id: Option<Guid>,
    name: String,
}

impl From<FileRow> for File {
    fn from(value: FileRow) -> Self {
        File::new(
            Some(value.id.into()),
            value.parent_id,
            FileSystemItemName::new_unchecked(value.name.clone()),
        )
    }
}

pub struct SqliteFileRepository {
    pool: Arc<SqlitePool>,
    tx: Arc<Mutex<Option<Transaction<'static, Sqlite>>>>,
}

impl SqliteFileRepository {
    pub fn new(
        pool: Arc<SqlitePool>,
        tx: Arc<Mutex<Option<Transaction<'static, Sqlite>>>>,
    ) -> Self {
        Self { pool, tx }
    }
}

#[async_trait]
impl FileRepository for SqliteFileRepository {
    async fn get_by_id(&self, id: Guid) -> Result<File, RepositoryError> {
        let row = sqlx::query_as!(
            FileRow,
            r#"SELECT id as "id: _", parent_id as "parent_id: _", name FROM files WHERE id = $1"#,
            id
        )
        .fetch_one(&*self.pool)
        .await;

        match row {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(row) => Ok(row.into()),
        }
    }

    async fn get_all_files(&self) -> Result<Vec<File>, RepositoryError> {
        let rows = sqlx::query_as!(
            FileRow,
            r#"SELECT id as "id: _", parent_id as "parent_id: _", name FROM files"#,
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
            r#"SELECT COUNT(*) FROM files WHERE parent_id = $1 AND name = $2"#,
            parent_id,
            name_string
        )
        .bind(name.to_string())
        .fetch_one(&*self.pool)
        .await;

        match row {
            Ok(cnt) => Ok(cnt > 0),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn create(&self, file: &File) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut().unwrap();

        let file_id = file.id();
        let file_name = file.name().to_string();
        let parent_id = file.parent_id();

        let result = sqlx::query!(
            "INSERT INTO files(id, name, parent_id) VALUES ($1, $2, $3)",
            file_id,
            file_name,
            parent_id
        )
        .execute(&mut *tx.as_mut())
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn update(&self, file: &File) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut().unwrap();

        let file_id = file.id();
        let file_name = file.name().to_string();
        let parent_id = file.parent_id();

        let result = sqlx::query!(
            "UPDATE files SET id = $1, name = $2, parent_id = $3 WHERE id = $1",
            file_id,
            file_name,
            parent_id
        )
        .execute(&mut *tx.as_mut())
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn delete_by_id(&self, id: Guid) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut().unwrap();

        let result = sqlx::query!("DELETE FROM files WHERE id = $1", id)
            .execute(&mut *tx.as_mut())
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }
}
