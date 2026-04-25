pub mod cells;
pub mod common;
pub mod database;
pub mod file_system;
pub mod fsrs;
pub mod local_configurations;
pub mod settings;

pub type Guid = uuid::Uuid;

pub const ROOT_FOLDER_ID: Guid = uuid::uuid!("00000000-0000-0000-0000-000000000001");
pub const DEFAULT_FSRS_PROFILE_ID: Guid = uuid::uuid!("00000000-0000-0000-0000-000000000002");
