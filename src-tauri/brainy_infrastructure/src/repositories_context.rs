use async_trait::async_trait;
use brainy_core::file_system::folder_repository::FolderRepository;

#[async_trait]
pub trait RepositoriesContext: Send + Sync {
    fn folder_repository(&mut self) -> Box<&mut dyn FolderRepository>;
    async fn start(&mut self);
    async fn commit(&mut self);
}

