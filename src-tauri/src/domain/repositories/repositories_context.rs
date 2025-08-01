use async_trait::async_trait;

use crate::domain::repositories::folder_repository::FolderRepository;

#[async_trait]
pub trait RepositoriesContext: Send + Sync {
    fn folder_repository(&self) -> Box<&dyn FolderRepository>;
    async fn start(&mut self);
    async fn commit(&mut self);
}
