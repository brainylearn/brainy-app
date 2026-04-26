use std::sync::Arc;

use injector_derive::ScopeInjectable;
use thiserror::Error;

use brainy_domain::{Guid, ROOT_FOLDER_ID, common::repository_error::RepositoryError};
use brainy_domain::{
    file_system::{
        repositories::folder_repository::FolderRepository,
        value_objects::fsrs_profile_choice::FsrsProfileChoice,
    },
    fsrs::repositories::fsrs_repository::{DeleteFsrsRequest, FsrsRepository},
};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum FsrsServiceError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),

    #[error(
        "You cannot delete the last profile, please create another one before deleting the current one"
    )]
    CannotDeleteLastProfile,
}

#[derive(ScopeInjectable)]
pub struct FsrsService {
    folder_repository: Arc<dyn FolderRepository>,
    fsrs_repository: Arc<dyn FsrsRepository>,
}

impl FsrsService {
    pub async fn delete_by_id(&self, id: Guid) -> Result<(), FsrsServiceError> {
        let mut root = self.folder_repository.get_by_id(ROOT_FOLDER_ID).await?;

        let all_profiles = self.fsrs_repository.get_all_fsrs_profiles().await?;

        if all_profiles.len() == 1 {
            return Err(FsrsServiceError::CannotDeleteLastProfile);
        }

        if let FsrsProfileChoice::Id(root_profile_id) = root.fsrs_profile_choice()
            && id == root_profile_id
        {
            let new_profile = all_profiles.iter().find(|item| item.id() != id).unwrap();
            root.set_fsrs_profile_choice(FsrsProfileChoice::Id(new_profile.id()));
            self.folder_repository.update(&root).await?;
        }

        self.fsrs_repository
            .delete_by_id(DeleteFsrsRequest::new(id))
            .await?;

        Ok(())
    }
}

// TODO:
