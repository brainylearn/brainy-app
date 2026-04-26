// TODO: resort imports
pub mod ai_integration;
pub mod backend;
pub mod backup;
pub mod cells;
pub mod common;
pub mod file_system;
pub mod fsrs;
pub mod settings;
pub mod sync;

pub mod generated_code {
    include!(concat!(env!("OUT_DIR"), "/generated_code.rs"));
}
