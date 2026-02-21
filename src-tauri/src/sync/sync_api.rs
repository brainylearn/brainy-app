use crate::{
    common::{api_error::ApiError, unit_of_work::UnitOfWorkExt},
    sync::sync_service::SyncService,
};
use injector::injector::Injector;
use tauri::State;

#[tauri::command]
pub async fn sync(injector: State<'_, Injector>) -> Result<(), ApiError> {
    let scope = injector.start_scope();

    scope
        .disable_foreign_key_constraint_for_current_transaction()
        .await?;

    let result = scope
        .resolve::<SyncService>()
        .await
        .sync_with_backend()
        .await;
    if let Err(err) = result {
        scope.rollback_changes().await?;
        return Err(err.into());
    }

    let result = scope.save_db_changes().await;
    if let Err(err) = result {
        scope.rollback_changes().await?;
        return Err(err.into());
    }

    Ok(())
}
