use std::sync::Arc;

use brainy_core::{
    common::traits::repositories_context::RepositoriesContext, sync::sync_service::SyncService,
};
use tauri::State;
use tokio::sync::Mutex;

use crate::api::ApiError;

#[tauri::command]
pub async fn sync(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    sync_service: State<'_, Arc<SyncService>>,
) -> Result<(), ApiError> {
    let mut context = context.lock().await;

    while sync_service.fetch_and_process_next_sync_page().await? {
        log::info!("Fetching and processing next sync page...");
        context.save_changes().await?;
    }

    context.save_changes().await?;
    Ok(())

    // TODO: send to server changes (all entities including deleted entitites) (exclude fetched
    // entities)
    // TODO: unit test repositories for filtering on modified date to send to backend
}
