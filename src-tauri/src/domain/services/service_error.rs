use thiserror::Error;

use crate::domain::repositories::repository_error::RepositoryError;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("An unknown error has happened!")]
    UnknownError,
    #[error("{0}")]
    UnknownRepositoryError(#[from] RepositoryError),
}
