use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

use crate::file_system::repositories::traits::{
    file_repository::FileRepository, folder_repository::FolderRepository,
};

#[derive(Debug, Error)]
pub enum RepositoriesContextError {
    #[error("An unknown error has happened!")]
    UnknownError(String),
    #[error("Transaction is already started!")]
    TransactionAlreadyStarted,
    #[error("Transaction is not started!")]
    TransactionNotStarted,
}

#[async_trait]
pub trait RepositoriesContext: Send + Sync {
    fn folder_repository(&self) -> Arc<dyn FolderRepository>;
    fn file_repository(&self) -> Arc<dyn FileRepository>;
    async fn start(&mut self) -> Result<(), RepositoriesContextError>;
    async fn commit(&mut self) -> Result<(), RepositoriesContextError>;
}
