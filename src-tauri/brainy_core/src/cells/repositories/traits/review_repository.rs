use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{cells::entities::review::Review, common::repository_error::RepositoryError};

#[async_trait]
pub trait ReviewRepository: Send + Sync {
    async fn create(&self, review: &Review) -> Result<(), RepositoryError>;
    async fn get_all_modified_on_or_after(
        &self,
        modified_date: DateTime<Utc>,
    ) -> Result<Vec<Review>, RepositoryError>;
}
