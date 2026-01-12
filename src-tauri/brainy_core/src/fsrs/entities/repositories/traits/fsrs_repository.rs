use async_trait::async_trait;

use crate::{
    Guid, common::repository_error::RepositoryError, fsrs::entities::fsrs_profile::FsrsProfile,
};

#[async_trait]
pub trait FsrsRepository: Send + Sync {
    async fn get_by_id(&self, id: Guid) -> Result<FsrsProfile, RepositoryError>;
    async fn get_all_fsrs_profiles(&self) -> Result<Vec<FsrsProfile>, RepositoryError>;
}
