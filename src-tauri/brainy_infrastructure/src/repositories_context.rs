use async_trait::async_trait;
use brainy_core::file_system::repositories::{
    file_repository::FileRepository, folder_repository::FolderRepository,
};

#[async_trait]
pub trait RepositoriesContext: Send + Sync {
    fn folder_repository(&self) -> Box<dyn FolderRepository>;
    fn file_repository(&self) -> Box<dyn FileRepository>;
    async fn start(&mut self);
    async fn commit(&mut self);
}
