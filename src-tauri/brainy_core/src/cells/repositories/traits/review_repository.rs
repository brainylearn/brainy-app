use async_trait::async_trait;

#[async_trait]
pub trait ReviewRepository: Send + Sync {}
