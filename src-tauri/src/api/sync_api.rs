use std::sync::Arc;

use brainy_core::{
    backend::traits::brainy_backend_client::BrainyBackendClient,
    common::traits::repositories_context::RepositoriesContext, sync::sync_service::SyncService,
};
use tauri::State;
use tokio::sync::Mutex;

use crate::api::ApiError;

#[tauri::command]
pub async fn sync(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    backend_client: State<'_, Box<dyn BrainyBackendClient>>,
    sync_service: State<'_, Arc<SyncService>>,
) -> Result<(), ApiError> {
    let mut context = context.lock().await;

    while sync_service
        .fetch_and_process_next_sync_page(&**backend_client)
        .await?
    {
        context.save_changes().await?;
    }

    context.save_changes().await?;
    Ok(())

    // TODO: send to server changes (all entities including deleted entitites) (exclude fetched
    // entities)
}
