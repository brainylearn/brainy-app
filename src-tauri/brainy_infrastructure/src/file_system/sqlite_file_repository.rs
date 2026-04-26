use std::sync::Arc;

use crate::common::db_transaction::DbTransaction;
use crate::file_system::file_row::FileRow;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use injector_derive::ScopeInjectable;

use brainy_domain::file_system::{
    entities::file::File, repositories::file_repository::FileRepository,
    value_objects::file_system_item_name::FileSystemItemName,
};
use brainy_domain::{Guid, common::repository_error::RepositoryError};

#[derive(ScopeInjectable)]
pub struct SqliteFileRepository {
    tx: Arc<DbTransaction>,
}

#[async_trait]
impl FileRepository for SqliteFileRepository {
    async fn get_by_id(&self, id: Guid) -> Result<File, RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let row = sqlx::query_as!(
            FileRow,
            r#"SELECT
                id as "id: _",
                created_date as "created_date: _",
                modified_date as "modified_date: _",
                parent_id as "parent_id: _",
                fsrs_profile_id as "fsrs_profile_id: _",
                name
            FROM files
            WHERE id = $1"#,
            id
        )
        .fetch_one(&mut *tx)
        .await;

        match row {
            Err(err) => Err(RepositoryError::Unknown(err.to_string())),
            Ok(row) => Ok(row.into()),
        }
    }

    async fn get_all_files(&self) -> Result<Vec<File>, RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let rows = sqlx::query_as!(
            FileRow,
            r#"SELECT
                id as "id: _",
                created_date as "created_date: _",
                modified_date as "modified_date: _",
                parent_id as "parent_id: _",
                fsrs_profile_id as "fsrs_profile_id: _",
                name
            FROM files"#,
        )
        .fetch_all(&mut *tx)
        .await;

        match rows {
            Err(err) => Err(RepositoryError::Unknown(err.to_string())),
            Ok(rows) => Ok(rows.into_iter().map(|row| row.into()).collect()),
        }
    }

    async fn get_folder_files(&self, parent_folder_id: Guid) -> Result<Vec<File>, RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let rows = sqlx::query_as!(
            FileRow,
            r#"SELECT
                id as "id: _",
                created_date as "created_date: _",
                modified_date as "modified_date: _",
                parent_id as "parent_id: _",
                fsrs_profile_id as "fsrs_profile_id: _",
                name
            FROM files
            WHERE parent_id = $1"#,
            parent_folder_id
        )
        .fetch_all(&mut *tx)
        .await;

        match rows {
            Err(err) => Err(RepositoryError::Unknown(err.to_string())),
            Ok(rows) => Ok(rows.into_iter().map(|row| row.into()).collect()),
        }
    }

    async fn get_all_modified_on_or_after(
        &self,
        modified_date: DateTime<Utc>,
    ) -> Result<Vec<File>, RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let rows = sqlx::query_as!(
            FileRow,
            r#"SELECT
                id as "id: _",
                created_date as "created_date: _",
                modified_date as "modified_date: _",
                parent_id as "parent_id: _",
                fsrs_profile_id as "fsrs_profile_id: _",
                name
            FROM files
            WHERE modified_date >= datetime($1)"#,
            modified_date
        )
        .fetch_all(&mut *tx)
        .await;

        match rows {
            Err(err) => Err(RepositoryError::Unknown(err.to_string())),
            Ok(rows) => Ok(rows.into_iter().map(|row| row.into()).collect()),
        }
    }

    async fn exists(
        &self,
        parent_id: Option<Guid>,
        name: &FileSystemItemName,
    ) -> Result<bool, RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let name_string = name.to_string();
        let row = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM files WHERE parent_id = $1 AND name = $2"#,
            parent_id,
            name_string
        )
        .fetch_one(&mut *tx)
        .await;

        match row {
            Ok(cnt) => Ok(cnt > 0),
            Err(err) => Err(RepositoryError::Unknown(err.to_string())),
        }
    }

    async fn create(&self, file: &File) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let file_id = file.id();
        let created_date = file.created_date();
        let file_name = file.name().to_string();
        let parent_id = file.parent_id();
        let modified_date = file.modified_date();
        let fsrs_profile_choice = Option::<Guid>::from(file.fsrs_profile_choice());

        let result = sqlx::query!(
            "INSERT INTO files(
                id,
                created_date,
                modified_date,
                name,
                parent_id,
                fsrs_profile_id)
            VALUES ($1, datetime($2), datetime($3), $4, $5, $6)",
            file_id,
            created_date,
            modified_date,
            file_name,
            parent_id,
            fsrs_profile_choice
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::Unknown(err.to_string())),
        }
    }

    async fn update(&self, file: &File) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let file_id = file.id();
        let created_date = file.created_date();
        let file_name = file.name().to_string();
        let parent_id = file.parent_id();
        let modified_date = file.modified_date();
        let fsrs_profile_choice = Option::<Guid>::from(file.fsrs_profile_choice());

        let result = sqlx::query!(
            "UPDATE files SET
                id = $1,
                created_date = datetime($2),
                modified_date = datetime($3),
                name = $4,
                parent_id = $5,
                fsrs_profile_id = $6
            WHERE id = $1",
            file_id,
            created_date,
            modified_date,
            file_name,
            parent_id,
            fsrs_profile_choice
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::Unknown(err.to_string())),
        }
    }

    async fn upsert_with_modified_date_if_modified_before(
        &self,
        file: &File,
        modified_date: DateTime<Utc>,
    ) -> Result<u64, RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let file_id = file.id();
        let file_name = file.name().to_string();
        let parent_id = file.parent_id();
        let created_date = file.created_date();
        let fsrs_profile_choice = Option::<Guid>::from(file.fsrs_profile_choice());

        let result = sqlx::query!(
            r#"INSERT INTO files(
                id,
                name,
                parent_id,
                modified_date,
                created_date,
                fsrs_profile_id)
            VALUES ($1, $2, $3, datetime($4), datetime($5), $6)
            ON CONFLICT(id) DO UPDATE
            SET id = $1,
                name = $2,
                parent_id = $3,
                modified_date = datetime($4),
                created_date = datetime($5),
                fsrs_profile_id = $6
            WHERE modified_date <= datetime($4)
            "#,
            file_id,
            file_name,
            parent_id,
            modified_date,
            created_date,
            fsrs_profile_choice
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(result) => Ok(result.rows_affected()),
            Err(err) => Err(RepositoryError::Unknown(err.to_string())),
        }
    }

    async fn delete_by_id(&self, id: Guid) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let result = sqlx::query!("DELETE FROM files WHERE id = $1", id)
            .execute(&mut *tx)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(RepositoryError::Unknown(err.to_string())),
        }
    }
}
