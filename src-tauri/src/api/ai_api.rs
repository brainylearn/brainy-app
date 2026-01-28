use std::sync::Arc;

use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use serde::Serialize;
use tauri::{State, ipc::Channel};
use tokio_stream::StreamExt;

use crate::api::ApiError;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum StreamLlmResponseEvent {
    InProgress(String),
    Finished,
    Error(String),
}

#[tauri::command]
pub async fn stream_ai_response(
    ollama: State<'_, Arc<Ollama>>,
    on_event: Channel<StreamLlmResponseEvent>,
    prompt: String,
) -> Result<(), ApiError> {
    // TODO: make it into a configuration
    let model = "ministral-3:14b".to_string();

    let mut stream = match ollama
        .generate_stream(GenerationRequest::new(model, prompt))
        .await
    {
        Ok(stream) => stream,
        Err(err) => return Err(ApiError(err.to_string())),
    };

    while let Some(res) = stream.next().await {
        let responses = match res {
            Ok(responses) => responses,
            Err(err) => {
                on_event.send(StreamLlmResponseEvent::Error(err.to_string()))?;
                break;
            }
        };
        for response in responses {
            on_event
                .send(StreamLlmResponseEvent::InProgress(response.response))
                .unwrap();
        }
    }

    on_event.send(StreamLlmResponseEvent::Finished)?;

    Ok(())
}
