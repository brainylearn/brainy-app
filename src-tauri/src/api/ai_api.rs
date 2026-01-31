use std::sync::Arc;

use brainy_core::ai_integration::ai_service::{AiService, StreamLlmResponseEvent};
use tauri::{State, ipc::Channel};

use crate::api::ApiError;

#[tauri::command]
pub async fn stream_ai_response(
    ai_service: State<'_, Arc<AiService>>,
    on_event: Channel<StreamLlmResponseEvent>,
    prompt: String,
) -> Result<(), ApiError> {
    let result = ai_service
        .stream(prompt, |event| match on_event.send(event) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        })
        .await;

    match result {
        Ok(()) => Ok(()),
        Err(err) => Err(ApiError::new(err.to_string())),
    }
}
