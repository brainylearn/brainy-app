pub mod multi_completion_model;
pub mod multi_response;
pub mod multi_streaming_response;

use rig::{client::CompletionClient, providers::ollama};

use crate::ai_integration::clients::multi_completion_client::multi_completion_model::MultiCompletionModel;

pub enum MultiCompletionClient {
    Ollama(ollama::Client),
}

impl CompletionClient for MultiCompletionClient {
    type CompletionModel = MultiCompletionModel;
}
