use thiserror::Error;

// TODO: move
#[derive(Debug, Error)]
pub enum RepositoriesContextError {
    #[error("An unknown error has happened!")]
    UnknownError(String),
}
