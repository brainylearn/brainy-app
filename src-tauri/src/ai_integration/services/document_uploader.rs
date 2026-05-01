use std::path::PathBuf;

use async_trait::async_trait;
use rig::{
    embeddings::{EmbedError, EmbeddingError},
    loaders::file::FileLoaderError,
    loaders::pdf::PdfLoaderError,
    vector_store::VectorStoreError,
};
use thiserror::Error;

use crate::{
    Guid, ai_integration::services::ai_client_provider::AiClientProviderError,
    common::repository_error::RepositoryError,
};

#[derive(Error, Debug)]
pub enum DocumentUploaderError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error("An unknown error has happened!")]
    Unknown(String),
    #[error("Error loading file: {0}")]
    FileLoader(#[from] FileLoaderError),
    #[error("Error loading pdf file: {0}")]
    PdfLoader(#[from] PdfLoaderError),
    #[error("Embed error: {0}")]
    Embed(#[from] EmbedError),
    #[error("Embedding error: {0}")]
    Embedding(#[from] EmbeddingError),
    #[error(transparent)]
    VectorStore(#[from] VectorStoreError),
    #[error(transparent)]
    AiClientProvider(#[from] AiClientProviderError),
}

impl From<String> for DocumentUploaderError {
    fn from(value: String) -> Self {
        DocumentUploaderError::Unknown(value)
    }
}

#[async_trait]
pub trait DocumentUploader: Send + Sync {
    async fn upload_document(
        &self,
        path: PathBuf,
        chat_id: Guid,
    ) -> Result<(), DocumentUploaderError>;
}
