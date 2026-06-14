use std::sync::Arc;

use chrono::{DateTime, Utc};
use injector::injector::Injector;
use tauri::State;

use crate::{
    Guid,
    common::api_error::ApiError,
    incremental_reading::{
        extracts::{
            entities::extract::ExtractStatus, repositories::extract_repository::ExtractRepository,
        },
        scheduling::{
            entities::incremental_reading_schedule::IncrementalReadingSchedule,
            repositories::incremental_reading_schedule_repository::IncrementalReadingScheduleRepository,
        },
    },
    infrastructure::extensions::unit_of_work::UnitOfWorkExt,
};

#[tauri::command]
pub async fn get_incremental_reading_schedule(
    injector: State<'_, Arc<Injector>>,
    cell_id: Guid,
) -> Result<Option<IncrementalReadingSchedule>, ApiError> {
    let scope = injector.start_scope();
    let result = scope
        .resolve::<dyn IncrementalReadingScheduleRepository>()
        .await
        .get_by_cell_id(cell_id)
        .await?;
    Ok(result)
}

#[tauri::command]
pub async fn get_pending_extracts_count(
    injector: State<'_, Arc<Injector>>,
    cell_id: Guid,
) -> Result<usize, ApiError> {
    let scope = injector.start_scope();
    let count = scope
        .resolve::<dyn ExtractRepository>()
        .await
        .count_by_cell_id_and_status(cell_id, &ExtractStatus::Pending)
        .await?;
    Ok(count as usize)
}

#[tauri::command]
pub async fn schedule_incremental_reading_later(
    injector: State<'_, Arc<Injector>>,
    cell_id: Guid,
    next_reading_date: DateTime<Utc>,
) -> Result<(), ApiError> {
    let scope = injector.start_scope();
    let repo = scope
        .resolve::<dyn IncrementalReadingScheduleRepository>()
        .await;

    let mut schedule = repo
        .get_by_cell_id(cell_id)
        .await?
        .ok_or_else(|| ApiError::new("Incremental reading schedule not found".to_string()))?;

    schedule.set_next_reading_date(next_reading_date);
    repo.update(&schedule).await?;
    scope.save_changes().await?;

    Ok(())
}
