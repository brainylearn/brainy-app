use std::sync::Arc;

use crate::{api::ApiError, dto::home_statistics::HomeStatistics, service::review_service};
use brainy_core::{
    cells::{
        cell_service::CellService,
        entities::{repetition::Repetition, review::Rating},
    },
    common::traits::repositories_context::RepositoriesContext,
};
use sea_orm::DbConn;
use tauri::State;
use tokio::sync::Mutex;

// TODO:
#[tauri::command]
pub async fn get_home_statistics(
    db_conn: State<'_, Mutex<DbConn>>,
) -> Result<HomeStatistics, String> {
    let db_conn = db_conn.lock().await;
    review_service::get_home_statistics(&db_conn).await
}

#[tauri::command]
pub async fn register_review(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    cell_service: State<'_, CellService>,
    new_repetition: Repetition,
    rating: Rating,
    study_time: u32,
) -> Result<(), ApiError> {
    let mut context = context.lock().await;
    cell_service
        .register_review(new_repetition, rating, study_time)
        .await?;
    context.save_changes().await?;
    Ok(())
}
