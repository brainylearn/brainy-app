use std::sync::Arc;

use crate::{
    Guid,
    common::repository_error::RepositoryError,
    file_system::{entities::file::File, value_objects::file_system_item_name::FileSystemItemName},
};
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

#[derive(sqlx::FromRow)]
struct FileRow {
    id: Guid,
    parent_id: Option<Guid>,
    name: String,
}

impl From<FileRow> for File {
    fn from(value: FileRow) -> Self {
        File::new_unchecked(
            Some(value.id.into()),
            value.parent_id,
            FileSystemItemName::new_unchecked(value.name),
        )
    }
}

impl From<&FileRow> for File {
    fn from(value: &FileRow) -> Self {
        File::new_unchecked(
            Some(value.id.into()),
            value.parent_id,
            FileSystemItemName::new_unchecked(value.name.clone()),
        )
    }
}

pub struct FileRepository {
    pool: Arc<SqlitePool>,
    tx: Arc<Mutex<Option<Transaction<'static, Sqlite>>>>,
}

impl FileRepository {
    pub fn new(
        pool: Arc<SqlitePool>,
        tx: Arc<Mutex<Option<Transaction<'static, Sqlite>>>>,
    ) -> Self {
        Self { pool, tx }
    }

    pub async fn get_by_id(&self, id: Guid) -> Result<File, RepositoryError> {
        let row = sqlx::query_as::<_, FileRow>("SELECT * FROM files WHERE id = $1")
            .bind(id)
            .fetch_one(&*self.pool)
            .await;

        match row {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(row) => Ok(row.into()),
        }
    }

    pub async fn get_all_files(&self) -> Result<Vec<File>, RepositoryError> {
        let rows = sqlx::query_as::<_, FileRow>("SELECT * FROM files")
            .fetch_all(&*self.pool)
            .await;

        match rows {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(rows) => Ok(rows.into_iter().map(|row| row.into()).collect()),
        }
    }

    pub async fn exists(
        &self,
        parent_id: Option<Guid>,
        name: &FileSystemItemName,
    ) -> Result<bool, RepositoryError> {
        let row = sqlx::query_as::<_, (bool,)>(
            "SELECT EXISTS (SELECT * FROM files WHERE parent_id = $1 AND name = $2)",
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

    pub async fn create(&self, file: &File) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut().unwrap();

        let result = sqlx::query("INSERT INTO files(id, name, parent_id) VALUES ($1, $2, $3)")
            .bind(file.id())
            .bind(file.name().to_string())
            .bind(file.parent_id())
            .execute(&mut *tx.as_mut())
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    pub async fn update(&self, file: &File) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut().unwrap();

        let result =
            sqlx::query("UPDATE files SET id = $1, name = $2, parent_id = $3 WHERE id = $1")
                .bind(file.id())
                .bind(file.name().to_string())
                .bind(file.parent_id())
                .execute(&mut *tx.as_mut())
                .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    pub async fn delete_by_id(&self, id: Guid) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut().unwrap();

        let result = sqlx::query("DELETE FROM files WHERE id = $1")
            .bind(id)
            .execute(&mut *tx.as_mut())
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }
}
