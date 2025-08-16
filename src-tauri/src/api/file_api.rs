// TODO: rename file to file_system api
use std::error::Error;

use brainy_infrastructure::repositories_context::RepositoriesContext;
use sea_orm::DbConn;
use serde::Serialize;
use tauri::State;
use tokio::sync::Mutex;

use crate::{dto::file_with_repetitions_count::FileWithRepetitionsCount, service::file_service};

// TODO: move
#[derive(Serialize)]
pub struct ApiError(String);

impl<T> From<T> for ApiError
where
    T: Error,
{
    fn from(value: T) -> Self {
        log::error!("An error occured: {:#?}", value);
        ApiError(value.to_string())
    }
}

#[tauri::command]
pub async fn get_files(
    context: State<'_, Mutex<Box<dyn RepositoriesContext>>>,
) -> Result<Vec<FileWithRepetitionsCount>, ApiError> {
    let mut context = context.lock().await;
    let folders = context
        .folder_repository()
        .get_all_files()
        .await?;

    Ok(folders.into_iter().map(|folder| folder.into()).collect())
}

#[tauri::command]
pub async fn create_folder(
    context: State<'_, Mutex<Box<dyn RepositoriesContext>>>,
    path: String,
) -> Result<uuid::fmt::Hyphenated, ApiError> {
    // TODO: move to service

    Ok(uuid::Uuid::new_v4().into())
}

#[tauri::command]
pub async fn create_file(db_conn: State<'_, Mutex<DbConn>>, path: String) -> Result<i32, String> {
    let db_conn = db_conn.lock().await;
    file_service::create_file(&*db_conn, path).await
}

#[tauri::command]
pub async fn delete_file(db_conn: State<'_, Mutex<DbConn>>, file_id: i32) -> Result<(), String> {
    let db_conn = db_conn.lock().await;
    file_service::delete_file(&db_conn, file_id).await
}

#[tauri::command]
pub async fn delete_folder(
    db_conn: State<'_, Mutex<DbConn>>,
    folder_id: i32,
) -> Result<(), String> {
    let db_conn = db_conn.lock().await;
    file_service::delete_folder(&db_conn, folder_id).await
}

#[tauri::command]
pub async fn move_file(
    db_conn: State<'_, Mutex<DbConn>>,
    file_id: i32,
    destination_folder_id: i32,
) -> Result<(), String> {
    let db_conn = db_conn.lock().await;
    file_service::move_file(&db_conn, file_id, destination_folder_id).await
}

#[tauri::command]
pub async fn move_folder(
    db_conn: State<'_, Mutex<DbConn>>,
    folder_id: i32,
    destination_folder_id: i32,
) -> Result<(), String> {
    let db_conn = db_conn.lock().await;
    file_service::move_folder(&db_conn, folder_id, destination_folder_id).await
}

#[tauri::command]
pub async fn rename_file(
    db_conn: State<'_, Mutex<DbConn>>,
    file_id: i32,
    new_name: String,
) -> Result<(), String> {
    let db_conn = db_conn.lock().await;
    file_service::rename_file(&db_conn, file_id, new_name).await
}

#[tauri::command]
pub async fn rename_folder(
    db_conn: State<'_, Mutex<DbConn>>,
    folder_id: i32,
    new_name: String,
) -> Result<(), String> {
    let db_conn = db_conn.lock().await;
    file_service::rename_folder(&db_conn, folder_id, new_name).await
}
