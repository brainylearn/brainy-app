use std::sync::Arc;

use rig::{
    agent::{Agent, MultiTurnStreamItem, Text},
    client::{CompletionClient, Nothing, ProviderClient},
    providers::ollama,
    streaming::{StreamedAssistantContent, StreamingPrompt},
};
use serde::Serialize;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

use crate::{
    ai_integration::{
        clients::multi_completion_client::{
            MultiCompletionClient, multi_completion_model::MultiCompletionModel,
        },
        tools::create_flash_card::CreateFlashCard,
    },
    settings::Settings,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum StreamLlmResponseEvent {
    InProgress(String),
    Finished,
    Error(String),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum AiServiceError {
    #[error("Ai is not enabled in settings!")]
    AiNotEnabled,
    #[error("Ollama model name is not filled in settings!")]
    OllamaModelNameIsNotFilled,
    #[error("An unknown error has happened!")]
    UnknownError(String),
}

impl From<String> for AiServiceError {
    fn from(value: String) -> Self {
        AiServiceError::UnknownError(value)
    }
}

pub struct AiService {
    settings: Arc<Mutex<Settings>>,
}

// TODO: unit test
impl AiService {
    pub fn new(settings: Arc<Mutex<Settings>>) -> Self {
        Self { settings }
    }

    pub async fn stream<F>(&self, prompt: String, on_event: F) -> Result<(), AiServiceError>
    where
        F: Fn(StreamLlmResponseEvent) -> Result<(), String>,
    {
        let agent = self.get_agent().await?;
        let mut stream = agent.stream_prompt(prompt).await;

        while let Some(content) = stream.next().await {
            match content {
                Ok(content) => {
                    if let MultiTurnStreamItem::StreamAssistantItem(
                        StreamedAssistantContent::Text(Text { text }),
                    ) = content
                    {
                        on_event(StreamLlmResponseEvent::InProgress(text))?;
                    }
                }
                Err(err) => {
                    on_event(StreamLlmResponseEvent::Error(err.to_string()))?;
                    break;
                }
            };
        }

        on_event(StreamLlmResponseEvent::Finished)?;

        Ok(())
    }

    async fn get_agent(&self) -> Result<Agent<MultiCompletionModel>, AiServiceError> {
        let settings = self.settings.lock().await;
        if !settings.enable_ai {
            return Err(AiServiceError::AiNotEnabled);
        }

        let (multi_client, model_name) = match settings.ollama_model_name {
            Some(ref model_name) => {
                log::info!("Using the Ollama model with name '{model_name}'.");
                (
                    MultiCompletionClient::Ollama(ollama::Client::from_val(Nothing)),
                    model_name,
                )
            }
            None => return Err(AiServiceError::OllamaModelNameIsNotFilled),
        };

        Ok(multi_client
            .agent(model_name)
            .temperature(0.5f64)
            // TODO: add it conditionally
            .tool(CreateFlashCard)
            .build())
    }
}
