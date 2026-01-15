use std::sync::Arc;

use brainy_core::{
    Guid,
    common::{
        repository_error::RepositoryError, traits::repositories_context::RepositoriesContext,
    },
    file_system::value_objects::fsrs_profile_choice::FsrsProfileChoice,
    fsrs::{entities::fsrs_profile::FsrsProfile, fsrs_service::FsrsService},
};
use tauri::State;
use tokio::sync::Mutex;

use crate::api::ApiError;

#[tauri::command]
pub async fn get_all_fsrs_profiles(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
) -> Result<Vec<FsrsProfile>, ApiError> {
    let context = context.lock().await;
    let result = context.fsrs_repository().get_all_fsrs_profiles().await?;
    Ok(result)
}

#[tauri::command]
pub async fn get_file_fsrs_profile(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    id: Guid,
) -> Result<FsrsProfile, ApiError> {
    let context = context.lock().await;
    let file = context.file_repository().get_by_id(id).await?;
    let result = get_fsrs_profile_recursively_for_item(
        &*context,
        *file.fsrs_profile_choice(),
        file.parent_id(),
    )
    .await?;
    Ok(result)
}

#[tauri::command]
pub async fn get_folder_fsrs_profile(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    id: Guid,
) -> Result<FsrsProfile, ApiError> {
    let context = context.lock().await;
    let folder = context.folder_repository().get_by_id(id).await?;
    let result = get_fsrs_profile_recursively_for_item(
        &*context,
        *folder.fsrs_profile_choice(),
        folder.parent_id(),
    )
    .await?;
    Ok(result)
}

#[tauri::command]
pub async fn get_fsrs_profile_choice_for_folder(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    id: Guid,
) -> Result<FsrsProfileChoice, ApiError> {
    let context = context.lock().await;
    let folder = context.folder_repository().get_by_id(id).await?;
    Ok(*folder.fsrs_profile_choice())
}

#[tauri::command]
pub async fn get_fsrs_profile_choice_for_file(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    id: Guid,
) -> Result<FsrsProfileChoice, ApiError> {
    let context = context.lock().await;
    let file = context.file_repository().get_by_id(id).await?;
    Ok(*file.fsrs_profile_choice())
}

#[tauri::command]
pub async fn get_parent_fsrs_profile_for_folder(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    id: Guid,
) -> Result<FsrsProfile, ApiError> {
    let context = context.lock().await;
    let folder = context.folder_repository().get_by_id(id).await?;
    let parent = context
        .folder_repository()
        .get_by_id(folder.parent_id().unwrap())
        .await?;
    let result = get_fsrs_profile_recursively_for_item(
        &*context,
        *parent.fsrs_profile_choice(),
        parent.parent_id(),
    )
    .await?;
    Ok(result)
}

#[tauri::command]
pub async fn get_parent_fsrs_profile_for_file(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    id: Guid,
) -> Result<FsrsProfile, ApiError> {
    let context = context.lock().await;
    let file = context.file_repository().get_by_id(id).await?;
    let parent = context
        .folder_repository()
        .get_by_id(file.parent_id().unwrap())
        .await?;
    let result = get_fsrs_profile_recursively_for_item(
        &*context,
        *parent.fsrs_profile_choice(),
        parent.parent_id(),
    )
    .await?;
    Ok(result)
}

// TODO: unit test
async fn get_fsrs_profile_recursively_for_item(
    context: &dyn RepositoriesContext,
    mut fsrs_profile_choice: FsrsProfileChoice,
    mut parent_id: Option<Guid>,
) -> Result<FsrsProfile, RepositoryError> {
    while FsrsProfileChoice::Inherit == fsrs_profile_choice {
        let parent = context
            .folder_repository()
            .get_by_id(parent_id.unwrap())
            .await?;
        fsrs_profile_choice = *parent.fsrs_profile_choice();
        parent_id = parent.parent_id();
    }

    if let FsrsProfileChoice::Id(id) = fsrs_profile_choice {
        let result = context.fsrs_repository().get_by_id(id).await?;
        return Ok(result);
    }

    unreachable!()
}

#[tauri::command]
pub async fn create_profile(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    name: String,
    request_retention: f64,
    maximum_interval: f64,
    weights: Vec<f64>,
) -> Result<FsrsProfile, ApiError> {
    let profile = FsrsProfile::new(None, name, request_retention, maximum_interval, weights);
    let context = context.lock().await;
    context.fsrs_repository().create(&profile).await?;
    context.save_changes().await?;
    Ok(profile)
}

#[tauri::command]
pub async fn update_profile(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    id: Guid,
    name: String,
    request_retention: f64,
    maximum_interval: f64,
    weights: Vec<f64>,
) -> Result<(), ApiError> {
    let profile = FsrsProfile::new(Some(id), name, request_retention, maximum_interval, weights);
    let context = context.lock().await;
    context.fsrs_repository().update(&profile).await?;
    context.save_changes().await?;
    Ok(())
}

#[tauri::command]
pub async fn set_fsrs_profile_choice_for_folder(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    id: Guid,
    fsrs_profile_choice: FsrsProfileChoice,
) -> Result<(), ApiError> {
    let context = context.lock().await;
    let mut folder = context.folder_repository().get_by_id(id).await?;
    folder.set_fsrs_profile_choice(fsrs_profile_choice);
    context.folder_repository().update(&folder).await?;
    context.save_changes().await?;
    Ok(())
}

#[tauri::command]
pub async fn set_fsrs_profile_choice_for_file(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    id: Guid,
    fsrs_profile_choice: FsrsProfileChoice,
) -> Result<(), ApiError> {
    let context = context.lock().await;
    let mut file = context.file_repository().get_by_id(id).await?;
    file.set_fsrs_profile_choice(fsrs_profile_choice);
    context.file_repository().update(&file).await?;
    context.save_changes().await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_fsrs_profile(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    fsrs_service: State<'_, Arc<FsrsService>>,
    id: Guid,
) -> Result<(), ApiError> {
    let context = context.lock().await;
    fsrs_service.delete_by_id(id).await?;
    context.save_changes().await?;
    Ok(())
}
