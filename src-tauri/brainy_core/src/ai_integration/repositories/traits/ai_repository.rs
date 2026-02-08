use async_trait::async_trait;

use crate::{
    Guid,
    ai_integration::entities::{chat::Chat, message::Message},
    common::repository_error::RepositoryError,
};

#[async_trait]
pub trait AiRepository: Send + Sync {
    async fn get_all_chats(&self) -> Result<Vec<Chat>, RepositoryError>;
    async fn upsert_chat(&self, chat: &Chat) -> Result<(), RepositoryError>;
    async fn get_by_id(&self, id: Guid) -> Result<Chat, RepositoryError>;
    async fn get_chat_messages(&self, id: Guid) -> Result<Vec<Message>, RepositoryError>;
    async fn delete_chat(&self, id: Guid) -> Result<(), RepositoryError>;
}
