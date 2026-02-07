use async_trait::async_trait;

use crate::{
    Guid, ai_integration::entities::chat::Chat, common::repository_error::RepositoryError,
};

#[async_trait]
pub trait AiRepository: Send + Sync {
    async fn upsert_chat(&self, chat: Chat) -> Result<(), RepositoryError>;
    async fn get_by_id(&self, id: Guid) -> Result<Chat, RepositoryError>;
}
