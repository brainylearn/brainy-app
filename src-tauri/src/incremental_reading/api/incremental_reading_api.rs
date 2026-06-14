use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use injector::injector::Injector;
use tauri::State;

use crate::{
    Guid,
    cells::{
        dto::create_cell_request_dto::CreateCellRequestDto, entities::cell::CellType,
        repositories::cell_repository::CellRepository, services::cell_creator::CellCreator,
        value_objects::incremental_reading::IncrementalReading,
    },
    common::api_error::ApiError,
    incremental_reading::{
        dto::pending_extract_dto::PendingExtractDto,
        extracts::{
            entities::extract::ExtractStatus, highlight_parser::parse_highlights,
            repositories::extract_repository::ExtractRepository,
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
pub async fn get_pending_extracts_with_content(
    injector: State<'_, Arc<Injector>>,
    cell_id: Guid,
) -> Result<Vec<PendingExtractDto>, ApiError> {
    let scope = injector.start_scope();

    let cell = scope
        .resolve::<dyn CellRepository>()
        .await
        .get_by_id(cell_id)
        .await?;

    let extracts = scope
        .resolve::<dyn ExtractRepository>()
        .await
        .get_by_cell_id(cell_id)
        .await?;

    let ir: IncrementalReading =
        serde_json::from_str(cell.content()).map_err(|e| ApiError::new(e.to_string()))?;

    let highlights: HashMap<String, String> = parse_highlights(&ir.content.unwrap_or_default())
        .into_iter()
        .map(|h| (h.id, h.inner_html))
        .collect();

    let result = extracts
        .into_iter()
        .filter(|e| e.status() == &ExtractStatus::Pending)
        .filter_map(|extract| {
            let inner_html = highlights.get(&extract.id().to_string())?.clone();
            Some(PendingExtractDto {
                id: extract.id().to_string(),
                inner_html,
            })
        })
        .collect();

    Ok(result)
}

#[tauri::command]
pub async fn update_extract_status(
    injector: State<'_, Arc<Injector>>,
    extract_id: Guid,
    status: ExtractStatus,
) -> Result<(), ApiError> {
    let scope = injector.start_scope();
    let repo = scope.resolve::<dyn ExtractRepository>().await;

    let mut extract = repo
        .get_by_id(extract_id)
        .await?
        .ok_or_else(|| ApiError::new("Extract not found".to_string()))?;

    extract.set_status(status);
    repo.update(&extract).await?;
    scope.save_changes().await?;

    Ok(())
}

#[tauri::command]
pub async fn create_cloze_from_extract(
    injector: State<'_, Arc<Injector>>,
    extract_id: Guid,
    cell_id: Guid,
    content: String,
) -> Result<(), ApiError> {
    let scope = injector.start_scope();

    let cell_repo = scope.resolve::<dyn CellRepository>().await;
    let file_id = cell_repo.get_by_id(cell_id).await?.file_id();
    let cell_index = cell_repo.get_number_of_cells_in_file(file_id).await?;

    scope
        .resolve::<dyn CellCreator>()
        .await
        .create_cell(CreateCellRequestDto {
            file_id,
            content,
            cell_type: CellType::Cloze,
            index: cell_index,
        })
        .await
        .map_err(|e| ApiError::new(e.to_string()))?;

    let extract_repo = scope.resolve::<dyn ExtractRepository>().await;
    let mut extract = extract_repo
        .get_by_id(extract_id)
        .await?
        .ok_or_else(|| ApiError::new("Extract not found".to_string()))?;

    extract.set_status(ExtractStatus::Added);
    extract_repo.update(&extract).await?;
    scope.save_changes().await?;

    Ok(())
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
