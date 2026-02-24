use std::sync::Arc;

use rig::{completion::ToolDefinition, tool::Tool};
use schemars::schema_for;
use serde::{Deserialize, Serialize};
use serde_json::to_value;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::{
    Guid,
    ai_integration::entities::message::{Message, MessageContent, ToolCall, ToolCallStatus},
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
}

pub struct CreateFlashCard {
    file_id: Guid,
    chat_id: Guid,
    messages_to_upsert: Arc<Mutex<Vec<Message>>>,
}

impl CreateFlashCard {
    pub fn new(file_id: Guid, chat_id: Guid, messages_to_upsert: Arc<Mutex<Vec<Message>>>) -> Self {
        Self {
            file_id,
            chat_id,
            messages_to_upsert,
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

        messages_to_upsert.push(Message::new(
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
        ));

        Ok("Request to create the flashcard has been presented to the user.".to_string())
    }
}
