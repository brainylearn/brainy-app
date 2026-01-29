use std::sync::Arc;

use brainy_core::ai_integration::ai_service::{AiService, StreamLlmResponseEvent};
use langchain_rust::schemas::Message;
use tauri::{State, ipc::Channel};

use crate::api::ApiError;

#[tauri::command]
pub async fn stream_ai_response(
    ai_service: State<'_, Arc<AiService>>,
    on_event: Channel<StreamLlmResponseEvent>,
    prompt: String,
) -> Result<(), ApiError> {
    let result = ai_service
        .stream(
            &[Message::new_human_message(prompt)],
            |event| match on_event.send(event) {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_string()),
            },
        )
        .await;

    match result {
        Ok(()) => Ok(()),
        Err(err) => Err(ApiError::new(err.to_string())),
    }
}

#[tauri::command]
pub async fn generate_ai_response(
    ai_service: State<'_, Arc<AiService>>,
    prompt: String,
) -> Result<String, ApiError> {
    let result = ai_service
        .generate(&[Message::new_human_message(prompt)])
        .await;

    match result {
        Ok(result) => Ok(result),
        Err(err) => Err(ApiError::new(err.to_string())),
    }
}
