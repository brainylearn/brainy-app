use std::sync::Arc;

use crate::{
    api::ApiError,
    dto::update_cell_request::UpdateCellRequest,
    entity::cell::{self},
    service::cell_service,
};
use brainy_core::{
    Guid,
    cells::{
        cell_service::CellService,
        entities::cell::{Cell, CellType},
    },
    common::traits::repositories_context::RepositoriesContext,
};
use sea_orm::DbConn;
use tauri::State;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn get_file_cells_ordered_by_index(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    file_id: Guid,
) -> Result<Vec<Cell>, ApiError> {
    let context = context.lock().await;
    let result = context
        .cell_repository()
        .get_file_cells_ordered_by_index(file_id)
        .await?;
    Ok(result)
}

#[tauri::command]
pub async fn create_cell(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    cell_service: State<'_, CellService>,
    file_id: Guid,
    content: String,
    cell_type: CellType,
    index: u32,
) -> Result<Guid, ApiError> {
    let mut context = context.lock().await;
    let id = cell_service
        .create_cell(file_id, content, cell_type, index)
        .await?;
    context.save_changes().await?;
    Ok(id)
}

#[tauri::command]
pub async fn delete_cell(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    cell_service: State<'_, CellService>,
    cell_id: Guid,
) -> Result<(), ApiError> {
    let mut context = context.lock().await;
    cell_service.delete_by_id(cell_id).await?;
    context.save_changes().await?;
    Ok(())
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
