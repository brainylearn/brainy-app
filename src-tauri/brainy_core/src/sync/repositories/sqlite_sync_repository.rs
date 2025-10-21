use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    cells::entities::{cell::Cell, repetition::Repetition, review::Review},
    common::repository_error::RepositoryError,
    file_system::entities::{file::File, folder::Folder},
    sync::{
        entities::deleted_entity::DeletedEntity,
        repositories::traits::sync_repository::SyncRepository,
    },
};

pub struct SqliteSyncRepository {
    pool: Arc<SqlitePool>,
    tx: Arc<Mutex<Transaction<'static, Sqlite>>>,
}

impl SqliteSyncRepository {
    pub fn new(pool: Arc<SqlitePool>, tx: Arc<Mutex<Transaction<'static, Sqlite>>>) -> Self {
        Self { pool, tx }
    }
}

#[async_trait]
impl SyncRepository for SqliteSyncRepository {
    async fn apply_deleted_entity(
        &self,
        deleted_entity: DeletedEntity,
    ) -> Result<u64, RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let DeletedEntity {
            entity_name,
            entity_id,
            entity_created_date,
            deleted_date,
        } = deleted_entity;

        let result = sqlx::query(&format!("DELETE FROM {entity_name} WHERE id = $1"))
            .bind(entity_id)
            .execute(&mut *tx)
            .await;

        if let Err(err) = result {
            return Err(RepositoryError::UnknownError(err.to_string()));
        }

        let result = sqlx::query!(
            r#"UPDATE deleted_entities
                SET deleted_date = datetime($1), entity_created_date = datetime($2)
                WHERE entity_name = $3 AND entity_id = $4
            "#,
            deleted_date,
            entity_created_date,
            entity_name,
            entity_id
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(result) => Ok(result.rows_affected()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn get_all_deleted_entities_on_or_after(
        &self,
        deleted_date: DateTime<Utc>,
    ) -> Result<Vec<DeletedEntity>, RepositoryError> {
        let rows = sqlx::query_as!(
            DeletedEntity,
            r#"SELECT
                entity_id as "entity_id: _",
                entity_name,
                entity_created_date as "entity_created_date: _",
                deleted_date as "deleted_date: _"
            FROM deleted_entities
            WHERE deleted_date >= datetime($1)"#,
            deleted_date
        )
        .fetch_all(&*self.pool)
        .await;

        match rows {
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
            Ok(rows) => Ok(rows.into_iter().collect()),
        }
    }

    async fn upsert_folder_with_modified_date_if_modified_before(
        &self,
        folder: &Folder,
        modified_date: DateTime<Utc>,
    ) -> Result<u64, RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let folder_id = folder.id();
        let folder_name = folder.name().to_string();
        let parent_id = folder.parent_id();
        let created_date = folder.created_date();
        let result = sqlx::query!(
            r#"INSERT INTO folders(
                id,
                name,
                parent_id,
                modified_date,
                created_date)
            VALUES ($1, $2, $3, datetime($4), datetime($5))
            ON CONFLICT(id) DO UPDATE
            SET id = $1, name = $2, parent_id = $3, modified_date = datetime($4), created_date = datetime($5)
            WHERE modified_date <= datetime($4)
            "#,
            folder_id,
            folder_name,
            parent_id,
            modified_date,
            created_date
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(result) => Ok(result.rows_affected()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn upsert_file_with_modified_date_if_modified_before(
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
        let result = sqlx::query!(
            r#"INSERT INTO files(
                id,
                name,
                parent_id,
                modified_date,
                created_date)
            VALUES ($1, $2, $3, datetime($4), datetime($5))
            ON CONFLICT(id) DO UPDATE
            SET id = $1, name = $2, parent_id = $3, modified_date = datetime($4), created_date = datetime($5)
            WHERE modified_date <= datetime($4)
            "#,
            file_id,
            file_name,
            parent_id,
            modified_date,
            created_date
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(result) => Ok(result.rows_affected()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn upsert_cell_without_repetition_and_with_modified_date_if_modified_before(
        &self,
        cell: &Cell,
        modified_date: DateTime<Utc>,
    ) -> Result<u64, RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();
        let id = cell.id();
        let content = cell.content();
        let cell_type = cell.cell_type();
        let file_id = cell.file_id();
        let index = cell.index();
        let searchable_content = cell.searchable_content();
        let created_date = cell.created_date();

        let result = sqlx::query!(
            r#"INSERT INTO cells(
                id,
                file_id,
                content,
                cell_type,
                cell_index,
                searchable_content,
                modified_date,
                created_date)
            VALUES ($1, $2, $3, $4, $5, $6, datetime($7), datetime($8))
            ON CONFLICT(id) DO UPDATE
            SET id = $1,
                file_id = $2,
                content = $3,
                cell_type = $4,
                cell_index = $5,
                searchable_content = $6,
                modified_date = datetime($7),
                created_date = datetime($8)
            WHERE modified_date <= datetime($7)"#,
            id,
            file_id,
            content,
            cell_type,
            index,
            searchable_content,
            modified_date,
            created_date
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(result) => Ok(result.rows_affected()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn upsert_repetition_with_modified_date_if_modified_before(
        &self,
        repetition: &Repetition,
        modified_date: DateTime<Utc>,
    ) -> Result<u64, RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let id = repetition.id();
        let file_id = repetition.file_id();
        let cell_id = repetition.cell_id();
        let due = repetition.due();
        let stability = repetition.stability();
        let difficulty = repetition.difficulty();
        let elapsed_days = repetition.elapsed_days();
        let scheduled_days = repetition.scheduled_days();
        let reps = repetition.reps();
        let lapses = repetition.lapses();
        let state = repetition.state();
        let last_review = repetition.last_review();
        let additional_content = repetition.additional_content();
        let created_date = repetition.created_date();

        let result = sqlx::query!(
            r#"INSERT INTO repetitions(
                id,
                file_id,
                cell_id,
                due,
                stability,
                difficulty,
                elapsed_days,
                scheduled_days,
                reps,
                lapses,
                state,
                last_review,
                additional_content,
                modified_date,
                created_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, datetime($14), datetime($15))
            ON CONFLICT(id) DO UPDATE SET
                file_id = $2,
                cell_id = $3,
                due = $4,
                stability = $5,
                difficulty = $6,
                elapsed_days = $7,
                scheduled_days = $8,
                reps = $9,
                lapses = $10,
                state = $11,
                last_review = $12,
                additional_content = $13,
                modified_date = datetime($14),
                created_date = datetime($15)
            WHERE modified_date <= datetime($14)
            "#,
            id,
            file_id,
            cell_id,
            due,
            stability,
            difficulty,
            elapsed_days,
            scheduled_days,
            reps,
            lapses,
            state,
            last_review,
            additional_content,
            modified_date,
            created_date
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(result) => Ok(result.rows_affected()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }

    async fn upsert_review_with_modified_date_if_modified_before(
        &self,
        review: &Review,
        modified_date: DateTime<Utc>,
    ) -> Result<u64, RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let id = review.id();
        let cell_id = review.cell_id();
        let study_time = review.study_time();
        let date = review.date();
        let rating = review.rating();
        let created_date = review.created_date();

        let result = sqlx::query!(
            r#"INSERT INTO reviews(
                id,
                cell_id,
                study_time,
                date,
                rating,
                modified_date,
                created_date) 
            VALUES ($1, $2, $3, $4, $5, datetime($6), datetime($7))
            ON CONFLICT(id) DO UPDATE SET
                id = $1,
                cell_id = $2,
                study_time = $3,
                date = $4,
                rating = $5,
                modified_date = datetime($6),
                created_date = datetime($7)
            WHERE modified_date <= datetime($6)
            "#,
            id,
            cell_id,
            study_time,
            date,
            rating,
            modified_date,
            created_date
        )
        .execute(&mut *tx)
        .await;

        match result {
            Ok(result) => Ok(result.rows_affected()),
            Err(err) => Err(RepositoryError::UnknownError(err.to_string())),
        }
    }
}
