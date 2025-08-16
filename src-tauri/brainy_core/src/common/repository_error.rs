use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("An unknown error has happened!")]
    UnknownError(String),
}
