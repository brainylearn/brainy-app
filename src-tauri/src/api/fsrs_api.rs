use std::{os::unix::process::parent_id, sync::Arc};

use brainy_core::{
    Guid,
    common::{
        repository_error::RepositoryError, traits::repositories_context::RepositoriesContext,
    },
    file_system::{entities::folder::Folder, value_objects::item_fsrs_profile::ItemFsrsProfile},
    fsrs::entities::fsrs_profile::FsrsProfile,
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
        file.fsrs_profile().clone(),
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
        folder.fsrs_profile().clone(),
        folder.parent_id(),
    )
    .await?;
    Ok(result)
}

// TODO: unit test
async fn get_fsrs_profile_recursively_for_item(
    context: &dyn RepositoriesContext,
    mut fsrs_profile: ItemFsrsProfile,
    mut parent_id: Option<Guid>,
) -> Result<FsrsProfile, RepositoryError> {
    while ItemFsrsProfile::Inherit == fsrs_profile {
        let parent = context
            .folder_repository()
            .get_by_id(parent_id.unwrap())
            .await?;
        fsrs_profile = parent.fsrs_profile().clone();
        parent_id = parent.parent_id();
    }

    if let ItemFsrsProfile::Id(id) = fsrs_profile {
        let result = context.fsrs_repository().get_by_id(id).await?;
        return Ok(result);
    }

    unreachable!()
}
