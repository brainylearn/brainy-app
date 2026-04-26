use std::sync::Arc;

use async_trait::async_trait;
use brainy_application::common::app_data_directory::AppDataDirectory;
use brainy_domain::settings::{
    entities::settings::Settings,
    repositories::settings_repository::{SettingsRepository, SettingsRepositoryError},
};
use injector_derive::ScopeInjectable;
use tokio::{
    fs::{self},
    io::AsyncReadExt,
    sync::Mutex,
};

#[cfg(not(debug_assertions))]
pub const SETTINGS_FILE_NAME: &str = "settings.json";
#[cfg(debug_assertions)]
pub const SETTINGS_FILE_NAME: &str = "settings.dev.json";

#[derive(ScopeInjectable)]
pub struct DiskSettingsRepository {
    // TODO: should not be public
    pub settings: Arc<Mutex<Settings>>,
    pub app_data_directory: Arc<AppDataDirectory>,
}

impl DiskSettingsRepository {
    pub async fn get_or_create_settings(
        app_data_directory: &AppDataDirectory,
        settings_if_non_existing: Settings,
    ) -> Result<Settings, SettingsRepositoryError> {
        if app_data_directory
            .get_path()
            .join(SETTINGS_FILE_NAME)
            .exists()
        {
            read_settings_from_file(app_data_directory).await
        } else {
            save_to_disk_inner(&settings_if_non_existing, app_data_directory).await?;
            Ok(settings_if_non_existing)
        }
    }
}

async fn read_settings_from_file(
    app_data_directory: &AppDataDirectory,
) -> Result<Settings, SettingsRepositoryError> {
    use tokio::fs::File;

    let settings_path = app_data_directory.get_path().join(SETTINGS_FILE_NAME);
    log::info!("Reading settings from '{SETTINGS_FILE_NAME}'.");
    let mut file = match File::open(settings_path).await {
        Err(err) => return Err(SettingsRepositoryError::ErrorOpeningFile(err.to_string())),
        Ok(file) => file,
    };
    let mut file_content = String::new();
    if let Err(err) = file.read_to_string(&mut file_content).await {
        return Err(SettingsRepositoryError::ErrorReadingFile(err.to_string()));
    }
    match serde_json::from_str(&file_content) {
        Ok(settings) => Ok(settings),
        Err(err) => Err(SettingsRepositoryError::Parsing(err.to_string())),
    }
}

#[async_trait]
impl SettingsRepository for DiskSettingsRepository {
    async fn get_settings(&self) -> Settings {
        self.settings.lock().await.clone()
    }

    async fn save_settings(&self, settings: Settings) -> Result<(), SettingsRepositoryError> {
        let mut current_settings = self.settings.lock().await;
        save_to_disk_inner(&settings, &self.app_data_directory).await?;
        *current_settings = settings;
        Ok(())
    }
}

async fn save_to_disk_inner(
    settings: &Settings,
    app_data_directory: &AppDataDirectory,
) -> Result<(), SettingsRepositoryError> {
    if let Err(err) = fs::create_dir_all(app_data_directory.get_path()).await {
        return Err(SettingsRepositoryError::Saving(err.to_string()));
    }

    let path = app_data_directory.get_path().join(SETTINGS_FILE_NAME);
    log::info!("Saving settings into '{}'.", path.to_str().unwrap());
    match fs::write(path, serde_json::to_string(settings).unwrap()).await {
        Ok(_) => Ok(()),
        Err(err) => Err(SettingsRepositoryError::Saving(err.to_string())),
    }
}

// TODO:
