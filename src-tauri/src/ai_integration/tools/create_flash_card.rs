use std::sync::Arc;

use async_trait::async_trait;
use rig::{completion::ToolDefinition, tool::Tool};
use schemars::schema_for;
use serde::{Deserialize, Serialize};
use serde_json::to_value;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::{
    Guid,
    ai_integration::{
        ai_service::{OnEventCallback, StreamLlmResponseEvent},
        entities::message::{Message, MessageContent, ToolCall, ToolCallStatus},
        tools::AcceptToolCall,
    },
    cells::{
        cell_service::CellService, entities::cell::CellType, models::flash_card::FlashCard,
        repositories::traits::cell_repository::CellRepository,
    },
    common::repository_error::RepositoryError,
};

#[derive(Deserialize, Debug, Serialize, schemars::JsonSchema)]
pub struct CreateFlashcardArgs {
    #[schemars(description = "The question shown to the user.'")]
    question: String,
    #[schemars(
        description = "The correct answer. Must be as concise as possible — a word, phrase, or single sentence."
    )]
    answer: String,
}

#[derive(Error, Debug)]
pub enum CreateFlashCardError {
    #[error("{0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("{0}")]
    UnknownRepositoryError(#[from] RepositoryError),
    #[error("{0}")]
    OnEvent(String),
}

// TODO: Unit test the two structs here
pub struct CreateFlashCard {
    file_id: Guid,
    chat_id: Guid,
    messages_to_upsert: Arc<Mutex<Vec<Message>>>,
    on_event: Option<OnEventCallback>,
}

impl CreateFlashCard {
    pub fn new(
        file_id: Guid,
        chat_id: Guid,
        messages_to_upsert: Arc<Mutex<Vec<Message>>>,
        on_event: Option<OnEventCallback>,
    ) -> Self {
        Self {
            file_id,
            chat_id,
            messages_to_upsert,
            on_event,
        }
    }
}

impl Tool for CreateFlashCard {
    const NAME: &'static str = "create_flashcard";

    type Error = CreateFlashCardError;
    type Args = CreateFlashcardArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        let parameters = serde_json::to_value(schema_for!(CreateFlashcardArgs)).unwrap();

        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Creates a single flashcard and adds it to the user's deck. \
                Call this tool once per card — never batch multiple facts into one call."
                .to_string(),
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        log::info!("{} called with arguments {:?}", Self::NAME, args);

        let mut messages_to_upsert = self.messages_to_upsert.lock().await;
        let message = Message::new(
            None,
            self.chat_id,
            MessageContent::ToolCall(ToolCall {
                id: Guid::new_v4().to_string(),
                name: Self::NAME.to_string(),
                display_name: "Create flashcard".to_string(),
                display_description_markdown: format!(
                    "\
                        **Question**: {}

\
                        **Answer**: {}",
                    args.question, args.answer
                )
                .to_string(),
                arguments: to_value(&args)?,
                status: ToolCallStatus::Pending,
                file_id: Some(self.file_id),
            }),
        );

        if let Some(on_event) = self.on_event.as_ref()
            && let Err(err) = on_event(StreamLlmResponseEvent::ToolCalled(message.clone()))
        {
            return Err(CreateFlashCardError::OnEvent(err));
        }

        messages_to_upsert.push(message);
        Ok("Request to create the flashcard has been presented to the user.".to_string())
    }
}

pub struct AcceptCreateFlashCard {
    cell_repository: Arc<dyn CellRepository>,
    cell_service: Arc<CellService>,
}

impl AcceptCreateFlashCard {
    pub fn new(cell_repository: Arc<dyn CellRepository>, cell_service: Arc<CellService>) -> Self {
        Self {
            cell_repository,
            cell_service,
        }
    }
}

#[async_trait]
impl AcceptToolCall for AcceptCreateFlashCard {
    type Args = CreateFlashcardArgs;

    // TODO: better error handling
    async fn accept_call(&self, tool_call: &ToolCall, args: Self::Args) -> Result<(), String> {
        if tool_call.file_id.is_none() {
            return Err("Missing file id!".to_string());
        }

        let cell_index = match self
            .cell_repository
            .get_number_of_cells_in_file(tool_call.file_id.unwrap())
            .await
        {
            Ok(cell_index) => cell_index,
            Err(err) => return Err(err.to_string()),
        };

        let flash_card = serde_json::to_string(&FlashCard {
            question: args.question,
            answer: args.answer,
        })
        .unwrap();

        log::info!(
            "Creating flash card with the following content {:?}",
            flash_card
        );

        self.cell_service
            .create_cell(
                tool_call.file_id.unwrap(),
                flash_card,
                CellType::FlashCard,
                cell_index,
            )
            .await
            .unwrap();

        Ok(())
    }
}
