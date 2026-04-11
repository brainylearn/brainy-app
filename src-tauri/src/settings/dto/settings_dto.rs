use serde::{Deserialize, Serialize};

use crate::settings::{entities::settings::Settings, value_objects::theme::Theme};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    pub database_location: String,
    pub database_directory: String,

    pub theme: Theme,
    pub zoom_percentage: f64,
    pub auto_sync: bool,

    pub enable_ai: bool,
    pub ollama_model_name: Option<String>,
    pub ollama_embeddings_model_name: Option<String>,
}

impl From<Settings> for SettingsDto {
    fn from(value: Settings) -> Self {
        Self {
            database_location: value.database_location().to_string(),
            database_directory: value.database_directory().to_string_lossy().to_string(),

            theme: value.theme,
            zoom_percentage: value.zoom_percentage,
            auto_sync: value.auto_sync,

            enable_ai: value.enable_ai,
            ollama_model_name: value.ollama_model_name,
            ollama_embeddings_model_name: value.ollama_embeddings_model_name,
        }
    }
}
