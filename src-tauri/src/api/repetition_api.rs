use std::sync::Arc;

use crate::api::ApiError;
use crate::entity::repetition;
use crate::service::repetition_service;
use crate::value_objects::file_repetitions_count::FileRepetitionCounts;
use brainy_core::Guid;
use brainy_core::cells::entities::repetition::Repetition;
use brainy_core::common::traits::repositories_context::RepositoriesContext;
use sea_orm::DbConn;
use tauri::State;
use tokio::sync::Mutex;

/// Returns the count of repetitions ready for study, i.e. their due is less
/// than or equal to now.
#[tauri::command]
pub async fn get_study_repetition_counts(
    db_conn: State<'_, Mutex<DbConn>>,
    file_id: i32,
) -> Result<FileRepetitionCounts, String> {
    let db_conn = db_conn.lock().await;
    repetition_service::get_study_repetition_counts(&db_conn, file_id).await
}

#[tauri::command]
pub async fn get_file_repetitions(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    file_id: Guid,
) -> Result<Vec<Repetition>, ApiError> {
    // TODO: shuffle
    let context = context.lock().await;
    let result = context
        .cell_repository()
        .get_file_repetitions(file_id)
        .await?;
    Ok(result)
}

#[tauri::command]
pub async fn get_repetitions_for_files(
    db_conn: State<'_, Mutex<DbConn>>,
    file_ids: Vec<i32>,
) -> Result<Vec<repetition::Model>, String> {
    let db_conn = db_conn.lock().await;
    repetition_service::get_repetitions_for_files(&db_conn, file_ids).await
}

#[tauri::command]
pub async fn reset_repetitions_for_cell(
    db_conn: State<'_, Mutex<DbConn>>,
    cell_id: i32,
) -> Result<(), String> {
    let db_conn = db_conn.lock().await;
    repetition_service::reset_repetitions_for_cell(&db_conn, cell_id).await
}
