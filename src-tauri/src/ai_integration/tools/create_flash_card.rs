use std::sync::Arc;

use injector::injector_scope::InjectorScope;
use rig::{completion::ToolDefinition, tool::Tool};
use schemars::schema_for;
use serde::{Deserialize, Serialize};
use serde_json::to_value;
use thiserror::Error;

use crate::{
    Guid,
    ai_integration::{
        entities::message::{Message, MessageContent, ToolCall, ToolCallStatus},
        repositories::traits::ai_repository::AiRepository,
    },
    common::repository_error::RepositoryError,
};

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
pub struct CreateFlashcardArgs {
    #[schemars(
        description = "The prompt, inquiry, or fill-in-the-blank statement presented to the user during a review."
    )]
    question: String,
    #[schemars(
        description = "The correct information or missing text required to satisfy the question."
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
    ai_repository: Arc<dyn AiRepository>,
}

impl CreateFlashCard {
    pub async fn new(file_id: Guid, chat_id: Guid, scope: &InjectorScope<'_>) -> Self {
        Self {
            file_id,
            chat_id,
            ai_repository: scope.resolve().await,
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
            description: "Generates a new study card to be added to the user's learning \
                collection. This tool only supports direct questions and does \
                not support cloze deletions or fill-in-the-blank formats in the \
                question."
                .to_string(),
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.ai_repository
            .upsert_message(&Message::new(
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
            ))
            .await?;

        Ok("Request to create the flashcard has been presented to the user.".to_string())
    }
}
