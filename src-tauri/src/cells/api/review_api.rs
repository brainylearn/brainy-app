use std::sync::Arc;

use crate::{common::api_error::ApiError, infrastructure::extensions::unit_of_work::UnitOfWorkExt};
use brainy_application::cells::cell_service::CellService;
use brainy_domain::cells::{
    entities::review::Rating, models::home_statistics::HomeStatistics,
    repositories::cell_repository::CellRepository,
    value_objects::repetition_update::RepetitionUpdate,
};
use injector::injector::Injector;
use tauri::State;

#[tauri::command]
pub async fn get_home_statistics(
    injector: State<'_, Arc<Injector>>,
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
    injector: State<'_, Arc<Injector>>,
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
    scope.save_changes().await?;
    Ok(())
}
