// TODO: move logging to services and file
use brainy_core::{
    common::domain_services::DomainServices, file_system::{value_objects::file_system_item_name::FileSystemItemName}, Guid
};
use brainy_infrastructure::repositories_context::RepositoriesContext;
use tauri::State;
use tokio::sync::Mutex;

use crate::{api::ApiError, dto::file_with_repetitions_count::FileWithRepetitionsCount};

#[tauri::command]
pub async fn get_files(
    context: State<'_, Mutex<Box<dyn RepositoriesContext>>>,
) -> Result<Vec<FileWithRepetitionsCount>, ApiError> {
    let context = context.lock().await;
    let folders = context.folder_repository().get_all_folders().await?;
    let files = context.file_repository().get_all_files().await?;

    // TODO: repetitions!
    let result = FileWithRepetitionsCount::parse_file_system(folders, files);

    Ok(result)
}

#[tauri::command]
pub async fn create_folder(
    context: State<'_, Mutex<Box<dyn RepositoriesContext>>>,
    services: State<'_, DomainServices>,
    parent_id: Option<Guid>,
    name: String,
) -> Result<Guid, ApiError> {
    let mut context = context.lock().await;
    context.start().await;

    let folder_id = services
        .file_system_service
        .create_folder(parent_id, FileSystemItemName::new(name)?)
        .await?;

    context.commit().await;

    log::info!("Created folder with id {folder_id}");
    Ok(folder_id)
}

#[tauri::command]
pub async fn create_file(
    context: State<'_, Mutex<Box<dyn RepositoriesContext>>>,
    services: State<'_, DomainServices>,
    parent_id: Option<Guid>,
    name: String,
) -> Result<Guid, ApiError> {
    let mut context = context.lock().await;
    context.start().await;

    let file_id = services
        .file_system_service
        .create_file(parent_id, FileSystemItemName::new(name)?)
        .await?;

    context.commit().await;

    log::info!("Created file with id {file_id}");
    Ok(file_id)
}

#[tauri::command]
pub async fn delete_file(
    context: State<'_, Mutex<Box<dyn RepositoriesContext>>>,
    file_id: Guid,
) ->Result<(), ApiError> {
    let mut context = context.lock().await;
    context.start().await;
    context.file_repository().delete_by_id(file_id).await?;
    context.commit().await;
    log::info!("Deleted file with id {file_id}");
    Ok(())
}

#[tauri::command]
pub async fn delete_folder(
    context: State<'_, Mutex<Box<dyn RepositoriesContext>>>,
    folder_id: Guid,
) -> Result<(), ApiError> {
    let mut context = context.lock().await;
    context.start().await;
    context.folder_repository().delete_by_id(folder_id).await?;
    context.commit().await;
    log::info!("Deleted folder with id {folder_id}");
    Ok(())
}

#[tauri::command]
pub async fn move_file(
    context: State<'_, Mutex<Box<dyn RepositoriesContext>>>,
    services: State<'_, DomainServices>,
    file_id: Guid,
    destination_folder_id: Option<Guid>,
) -> Result<(), ApiError> {
    let mut context = context.lock().await;
    context.start().await;
    services
        .file_system_service
        .move_file(file_id, destination_folder_id)
        .await?;
    context.commit().await;
    Ok(())
}

#[tauri::command]
pub async fn move_folder(
    context: State<'_, Mutex<Box<dyn RepositoriesContext>>>,
    services: State<'_, DomainServices>,
    folder_id: Guid,
    destination_folder_id: Option<Guid>,
) -> Result<(), ApiError> {
    let mut context = context.lock().await;
    context.start().await;
    services
        .file_system_service
        .move_folder(folder_id, destination_folder_id)
        .await?;
    context.commit().await;
    Ok(())
}

#[tauri::command]
pub async fn rename_file(
    context: State<'_, Mutex<Box<dyn RepositoriesContext>>>,
    services: State<'_, DomainServices>,
    file_id: Guid,
    new_name: String,
) -> Result<(), ApiError> {
    log::info!("Renaming file with id: {file_id}, and new name: {new_name}");

    let mut context = context.lock().await;
    context.start().await;
    services
        .file_system_service
        .rename_file(file_id, FileSystemItemName::new(new_name)?)
        .await?;
    context.commit().await;
    Ok(())
}

#[tauri::command]
pub async fn rename_folder(
    context: State<'_, Mutex<Box<dyn RepositoriesContext>>>,
    services: State<'_, DomainServices>,
    folder_id: Guid,
    new_name: String,
) -> Result<(), ApiError> {
    let mut context = context.lock().await;
    context.start().await;
    services
        .file_system_service
        .rename_folder(folder_id, FileSystemItemName::new(new_name)?)
        .await?;
    context.commit().await;
    Ok(())
}
