use crate::file_system::file_system_service::FileSystemService;

pub struct DomainServices {
    pub file_system_service: Box<dyn FileSystemService>,
}
