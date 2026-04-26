use std::sync::Arc;

use brainy_domain::{
    database::database_connection_manager::{
        DatabaseConnectionManager, DatabaseConnectionManagerError,
    },
    settings::{
        repositories::settings_repository::{SettingsRepository, SettingsRepositoryError},
        value_objects::settings_profile::SettingsProfile,
    },
};
use injector_derive::ScopeInjectable;
use thiserror::Error;

use crate::settings::dto::update_settings_request::UpdateSettingsRequest;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SettingsServiceError {
    #[error(transparent)]
    SettingsRepository(#[from] SettingsRepositoryError),
    #[error(transparent)]
    DatabaseConnectionManager(#[from] DatabaseConnectionManagerError),
}

#[derive(ScopeInjectable)]
pub struct SettingsService {
    settings_repository: Arc<dyn SettingsRepository>,
    database_connection_manager: Arc<dyn DatabaseConnectionManager>,
}

impl SettingsService {
    pub async fn update_settings(
        &self,
        new_settings: UpdateSettingsRequest,
    ) -> Result<(), SettingsServiceError> {
        let mut settings = self.settings_repository.get_settings().await;
        let mut change_database_location = false;

        if let Some(new_base_dir) = new_settings.base_database_directory
            && new_base_dir != settings.base_database_directory
        {
            settings.base_database_directory = new_base_dir;
            change_database_location = true;
        }
        if let Some(new_profile) = new_settings.profile
            && new_profile != settings.profile
        {
            settings.profile = new_profile;
            change_database_location = true;
        }
        if let Some(theme) = new_settings.theme {
            settings.theme = theme;
        }
        if let Some(zoom_percentage) = new_settings.zoom_percentage {
            settings.zoom_percentage = zoom_percentage;
        }
        if let Some(auto_sync) = new_settings.auto_sync {
            settings.auto_sync = auto_sync;
        }
        if let Some(enable_ai) = new_settings.enable_ai {
            settings.enable_ai = enable_ai;
        }
        if let Some(ollama_model_name) = new_settings.ollama_model_name {
            settings.ollama_model_name = ollama_model_name;
        }
        if let Some(ollama_embeddings_model_name) = new_settings.ollama_embeddings_model_name {
            settings.ollama_embeddings_model_name = ollama_embeddings_model_name;
        }

        if change_database_location {
            log::info!(
                "Changing database location to {}",
                settings.database_location()
            );
            self.database_connection_manager
                .connect_to_database(settings.database_location())
                .await?;
        }

        self.settings_repository.save_settings(settings).await?;

        Ok(())
    }

    /// Sets the profile for settings when the user is newly created, leading to
    /// database being moved to the new user location.
    pub async fn set_profile_for_new_user(
        &self,
        profile_name: String,
    ) -> Result<(), SettingsServiceError> {
        let mut settings = self.settings_repository.get_settings().await;
        settings.profile = SettingsProfile::User(profile_name);
        self.database_connection_manager
            .move_database_to(settings.database_location())
            .await?;
        self.settings_repository.save_settings(settings).await?;
        Ok(())
    }
}
