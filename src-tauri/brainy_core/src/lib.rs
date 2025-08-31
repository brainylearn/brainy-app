pub mod common;
pub mod file_system;

pub type Guid = uuid::Uuid;

pub const ROOT_FOLDER_ID: Guid = uuid::uuid!("00000000-0000-0000-0000-000000000001");
