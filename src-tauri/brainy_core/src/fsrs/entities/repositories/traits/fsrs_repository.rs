use async_trait::async_trait;

use crate::{
    Guid, common::repository_error::RepositoryError, fsrs::entities::fsrs_profile::FsrsProfile,
};

#[async_trait]
pub trait FsrsRepository: Send + Sync {
    async fn get_by_id(&self, id: Guid) -> Result<FsrsProfile, RepositoryError>;
    async fn get_all_fsrs_profiles(&self) -> Result<Vec<FsrsProfile>, RepositoryError>;
    async fn create(&self, fsrs_profile: &FsrsProfile) -> Result<(), RepositoryError>;
    async fn update(&self, fsrs_profile: &FsrsProfile) -> Result<(), RepositoryError>;
    async fn delete_by_id(&self, request: DeleteFsrsRequest) -> Result<(), RepositoryError>;
}

pub struct DeleteFsrsRequest {
    id: Guid,
}

impl DeleteFsrsRequest {
    pub(in crate::fsrs) fn new(id: Guid) -> Self {
        Self { id }
    }

    pub fn id(&self) -> Guid {
        self.id
    }
}
