use brainy_core::common::repository_error::RepositoryError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("An unknown error has happened!")]
    UnknownError,
    #[error("{0}")]
    UnknownRepositoryError(#[from] RepositoryError),
}
