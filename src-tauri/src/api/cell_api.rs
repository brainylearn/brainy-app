use std::sync::Arc;

use crate::{
    api::ApiError, dto::update_cell_request::UpdateCellRequest, entity::cell::{self, CellType}, service::cell_service
};
use brainy_core::{cells::entities::cell::Cell, common::traits::repositories_context::RepositoriesContext, Guid};
use sea_orm::DbConn;
use tauri::State;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn get_file_cells_ordered_by_index(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    file_id: Guid,
) -> Result<Vec<Cell>, ApiError> {
    let context = context.lock().await;
    let result = context.cell_repository().get_file_cells_ordered_by_index(file_id).await?;
    Ok(result)
}

#[tauri::command]
pub async fn create_cell(
    db_conn: State<'_, Mutex<DbConn>>,
    file_id: i32,
    content: String,
    cell_type: CellType,
    index: i32,
) -> Result<i32, String> {
    let db_conn = db_conn.lock().await;
    cell_service::create_cell(&db_conn, file_id, &content, cell_type, index).await
}

#[tauri::command]
pub async fn delete_cell(db_conn: State<'_, Mutex<DbConn>>, cell_id: i32) -> Result<(), String> {
    let db_conn = db_conn.lock().await;
    cell_service::delete_cell(&db_conn, cell_id).await
}

#[tauri::command]
pub async fn move_cell(
    db_conn: State<'_, Mutex<DbConn>>,
    cell_id: i32,
    new_index: i32,
) -> Result<(), String> {
    let db_conn = db_conn.lock().await;
    cell_service::move_cell(&db_conn, cell_id, new_index).await
}

#[tauri::command]
pub async fn update_cells_contents(
    db_conn: State<'_, Mutex<DbConn>>,
    requests: Vec<UpdateCellRequest>,
) -> Result<(), String> {
    let db_conn = db_conn.lock().await;
    cell_service::update_cells_contents(&db_conn, requests).await
}

#[tauri::command]
pub async fn get_cells_for_files(
    db_conn: State<'_, Mutex<DbConn>>,
    file_ids: Vec<i32>,
) -> Result<Vec<cell::Model>, String> {
    let db_conn = db_conn.lock().await;
    cell_service::get_cells_for_files(&db_conn, file_ids).await
}
