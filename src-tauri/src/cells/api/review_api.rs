use crate::{
    cells::{
        cell_service::CellService, entities::review::Rating,
        models::home_statistics::HomeStatistics,
        repositories::traits::cell_repository::CellRepository,
        value_objects::repetition_update::RepetitionUpdate,
    },
    common::{api_error::ApiError, unit_of_work::UnitOfWorkExt},
};
use injector::injector::Injector;
use tauri::State;

#[tauri::command]
pub async fn get_home_statistics(
    injector: State<'_, Injector>,
) -> Result<HomeStatistics, ApiError> {
    let scope = injector.start_scope();
    let result = scope
        .resolve::<dyn CellRepository>()
        .await
        .get_home_statistics()
        .await?;
    Ok(result)
}

#[tauri::command]
pub async fn register_review(
    injector: State<'_, Injector>,
    repetition_update: RepetitionUpdate,
    rating: Rating,
    study_time: u32,
) -> Result<(), ApiError> {
    let scope = injector.start_scope();
    scope
        .resolve::<CellService>()
        .await
        .register_review(repetition_update, rating, study_time)
        .await?;
    scope.save_db_changes().await?;
    Ok(())
}
