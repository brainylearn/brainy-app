use std::sync::Arc;

use rig::{
    agent::{Agent, MultiTurnStreamItem, StreamingError, Text},
    client::{CompletionClient, Nothing, ProviderClient},
    completion::PromptError,
    providers::ollama,
    streaming::{StreamedAssistantContent, StreamingChat},
};
use serde::Serialize;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

use crate::{
    Guid,
    ai_integration::{
        ai_state::AiState,
        clients::multi_completion_client::{
            MultiCompletionClient, multi_completion_model::MultiCompletionModel,
        },
        entities::chat::Chat,
        repositories::traits::ai_repository::AiRepository,
        state_cancellation_hook::StateCancellationHook,
        tools::create_flash_card::CreateFlashCard,
    },
    common::repository_error::RepositoryError,
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
    #[error("{0}")]
    UnknownRepositoryError(#[from] RepositoryError),
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
    ai_repository: Arc<dyn AiRepository>,
}

// TODO: unit test
impl AiService {
    pub fn new(
        settings: Arc<Mutex<Settings>>,
        state: Arc<AiState>,
        ai_repository: Arc<dyn AiRepository>,
    ) -> Self {
        Self {
            settings,
            state,
            ai_repository,
        }
    }

    pub async fn stream<F>(
        &self,
        prompt: String,
        chat_id: Option<Guid>,
        on_event: F,
    ) -> Result<(), AiServiceError>
    where
        F: Fn(StreamLlmResponseEvent) -> Result<(), String>,
    {
        let _ = self.state.start_generation().await;

        let chat;
        if let Some(chat_id) = chat_id {
            chat = self.ai_repository.get_by_id(chat_id).await?;
        } else {
            // TODO: name
            chat = Chat::new(None, "Test".into(), vec![]);
        }

        let messages = chat
            .messages()
            .iter()
            .map(|message| message.clone().into())
            .collect();

        let agent = self.get_agent().await?;
        let mut stream = agent
            .stream_chat(prompt, messages)
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

        // TODO: add user message and ai and save them to chat

        Ok(())
    }

    async fn get_agent(&self) -> Result<Agent<MultiCompletionModel>, AiServiceError> {
        let client = self.get_multi_completion_client().await?;
        let model_name = self.get_model_name().await;

        Ok(client
            .agent(model_name)
            .temperature(0.5f64)
            .tool(CreateFlashCard)
            .build())
    }

    async fn get_multi_completion_client(&self) -> Result<MultiCompletionClient, AiServiceError> {
        let settings = self.settings.lock().await;
        if !settings.enable_ai {
            return Err(AiServiceError::AiNotEnabled);
        }

        if settings.ollama_model_name.is_none() {
            return Err(AiServiceError::OllamaModelNameIsNotFilled);
        }

        let client = MultiCompletionClient::Ollama(ollama::Client::from_val(Nothing));
        Ok(client)
    }

    async fn get_model_name(&self) -> String {
        let settings = self.settings.lock().await;
        let model_name = settings.ollama_model_name.as_ref().unwrap().clone();
        log::info!("Using the model with name '{model_name}'.");
        model_name
    }
}
