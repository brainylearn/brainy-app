use std::sync::Arc;

use rig::{
    agent::{Agent, MultiTurnStreamItem, StreamingError, Text},
    client::{CompletionClient, Nothing, ProviderClient},
    completion::PromptError,
    providers::ollama,
    streaming::{StreamedAssistantContent, StreamingPrompt},
};
use serde::Serialize;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

use crate::{
    ai_integration::{
        ai_state::AiState,
        clients::multi_completion_client::{
            MultiCompletionClient, multi_completion_model::MultiCompletionModel,
        },
        state_cancellation_hook::StateCancellationHook,
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
    state: Arc<AiState>,
}

// TODO: unit test
impl AiService {
    pub fn new(settings: Arc<Mutex<Settings>>, state: Arc<AiState>) -> Self {
        Self { settings, state }
    }

    pub async fn stream<F>(&self, prompt: String, on_event: F) -> Result<(), AiServiceError>
    where
        F: Fn(StreamLlmResponseEvent) -> Result<(), String>,
    {
        let _ = self.state.start_generation().await;

        let agent = self.get_agent().await?;
        let mut stream = agent
            .stream_prompt(prompt)
            .with_hook(StateCancellationHook::new(self.state.clone()))
            .await;

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
                    let mut should_call_callback = true;

                    if let StreamingError::Prompt(ref prompt_error) = err
                        && matches!(**prompt_error, PromptError::PromptCancelled { .. })
                    {
                        should_call_callback = false;
                    }

                    if should_call_callback {
                        on_event(StreamLlmResponseEvent::Error(err.to_string()))?;
                    }
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

        if settings.ollama_model_name.is_none() {
            return Err(AiServiceError::OllamaModelNameIsNotFilled);
        }
        let model_name = settings.ollama_model_name.as_ref().unwrap();

        log::info!("Using the Ollama model with name '{model_name}'.");
        let multi_client = MultiCompletionClient::Ollama(ollama::Client::from_val(Nothing));

        Ok(multi_client
            .agent(model_name)
            .temperature(0.5f64)
            .tool(CreateFlashCard)
            .build())
    }
}
