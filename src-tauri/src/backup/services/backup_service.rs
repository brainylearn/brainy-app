use async_trait::async_trait;
use thiserror::Error;

use crate::{
    common::repository_error::RepositoryError,
    database::database_connection_manager::DatabaseConnectionManagerError,
};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum BackupServiceError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    DatabaseConnectionManager(#[from] DatabaseConnectionManagerError),
    #[error("The application is not able to list the entries in the settings folder!")]
    CannotListEntriesInFolder(String),
}

pub const TIME_BETWEEN_BACKUPS_IN_MINUTES: u64 = 120;

#[async_trait]
pub trait BackupService: Send + Sync {
    async fn ensure_backup(&self) -> Result<(), BackupServiceError>;
}
