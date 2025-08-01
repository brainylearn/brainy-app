use crate::{domain::{repositories::repositories_context::RepositoriesContext, value_objects::path::Path}, dto::file_with_repetitions_count::FileWithRepetitionsCount, service::file_service};
use sea_orm::DbConn;
use tauri::State;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn get_files(
    db_conn: State<'_, Mutex<DbConn>>,
    context: State<'_, Mutex<Box<dyn RepositoriesContext>>>,
) -> Result<Vec<FileWithRepetitionsCount>, String> {
    let mut context = context.lock().await;
    println!("{:#?}", context.folder_repository().get_by_path(&Path::new("")).await.unwrap().unwrap());

    let db_conn = db_conn.lock().await;
    file_service::get_files(&db_conn).await
}

#[tauri::command]
pub async fn create_folder(db_conn: State<'_, Mutex<DbConn>>, path: String) -> Result<i32, String> {
    let db_conn = db_conn.lock().await;
    file_service::create_folder(&*db_conn, path).await
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
