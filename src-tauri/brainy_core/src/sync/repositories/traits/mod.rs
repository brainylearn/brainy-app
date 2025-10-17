use async_trait::async_trait;

use crate::{Guid, common::repository_error::RepositoryError};

#[async_trait]
pub trait DeletedEntityRepository: Send + Sync {
    async fn apply_deleted_entity(
        &self,
        entity_name: &str,
        entity_id: Guid,
    ) -> Result<(), RepositoryError>;
}
