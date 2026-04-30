use async_trait::async_trait;
use thiserror::Error;

use crate::{
    cells::{entities::review::Rating, value_objects::repetition_update::RepetitionUpdate},
    common::repository_error::RepositoryError,
};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ReviewRegistrarError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[async_trait]
pub trait ReviewRegistrar: Send + Sync {
    async fn register_review(
        &self,
        repetition_update: RepetitionUpdate,
        rating: Rating,
        study_time: u32,
    ) -> Result<(), ReviewRegistrarError>;
}
