use std::sync::Arc;

use crate::{api::ApiError, dto::search_result::SearchResult};
use brainy_core::common::traits::repositories_context::RepositoriesContext;
use tauri::State;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn search_cells(
    context: State<'_, Arc<Mutex<dyn RepositoriesContext>>>,
    search_text: String,
) -> Result<SearchResult, ApiError> {
    let context = context.lock().await;
    let cells = context.cell_repository().search_cells(&search_text).await?;

    Ok(SearchResult {
        cells,
        repetitions: Vec::new(),
    })
}
