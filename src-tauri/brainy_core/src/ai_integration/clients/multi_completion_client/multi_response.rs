use rig::providers::ollama;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum MultiResponse {
    Ollama(ollama::CompletionResponse),
}

impl From<ollama::CompletionResponse> for MultiResponse {
    fn from(value: ollama::CompletionResponse) -> Self {
        MultiResponse::Ollama(value)
    }
}
