use std::sync::Arc;

use async_trait::async_trait;
use injector_derive::ScopeInjectable;

use crate::{
    Guid,
    common::repository_error::RepositoryError,
    incremental_reading::scheduling::{
        entities::incremental_reading_schedule::IncrementalReadingSchedule,
        repositories::incremental_reading_schedule_repository::IncrementalReadingScheduleRepository,
    },
    infrastructure::{
        repositories::sqlite::sqlite_rows::incremental_reading_schedule_row::IncrementalReadingScheduleRow,
        value_objects::db_transaction::DbTransaction,
    },
};

#[derive(ScopeInjectable)]
pub struct SqliteIncrementalReadingScheduleRepository {
    tx: Arc<DbTransaction>,
}

#[async_trait]
impl IncrementalReadingScheduleRepository for SqliteIncrementalReadingScheduleRepository {
    async fn get_by_cell_id(
        &self,
        cell_id: Guid,
    ) -> Result<Option<IncrementalReadingSchedule>, RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        let row = sqlx::query_as!(
            IncrementalReadingScheduleRow,
            r#"SELECT
                id as "id: _",
                created_date as "created_date: _",
                modified_date as "modified_date: _",
                cell_id as "cell_id: _",
                priority as "priority: _",
                title,
                is_finished as "is_finished: _",
                next_reading_date as "next_reading_date: _",
                has_extracts as "has_extracts: _"
            FROM incremental_reading_schedules
            WHERE cell_id = $1"#,
            cell_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        Ok(row.map(IncrementalReadingSchedule::from))
    }

    async fn create(&self, schedule: &IncrementalReadingSchedule) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        sqlx::query!(
            r#"INSERT INTO incremental_reading_schedules(
                id,
                created_date,
                modified_date,
                cell_id,
                priority,
                title,
                is_finished,
                next_reading_date,
                has_extracts)
            VALUES ($1, datetime($2), datetime($3), $4, $5, $6, $7, datetime($8), $9)"#,
            schedule.id(),
            schedule.created_date(),
            schedule.modified_date(),
            schedule.cell_id(),
            schedule.priority(),
            schedule.title(),
            schedule.is_finished(),
            schedule.next_reading_date(),
            schedule.has_extracts(),
        )
        .execute(&mut *tx)
        .await?;

        Ok(())
    }

    async fn update(&self, schedule: &IncrementalReadingSchedule) -> Result<(), RepositoryError> {
        let mut tx = self.tx.lock().await;
        let tx = tx.as_mut();

        sqlx::query!(
            r#"UPDATE incremental_reading_schedules
            SET priority = $1,
                title = $2,
                is_finished = $3,
                next_reading_date = datetime($4),
                has_extracts = $5
            WHERE id = $6"#,
            schedule.priority(),
            schedule.title(),
            schedule.is_finished(),
            schedule.next_reading_date(),
            schedule.has_extracts(),
            schedule.id(),
        )
        .execute(&mut *tx)
        .await?;

        Ok(())
    }
}
