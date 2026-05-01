use async_trait::async_trait;
use thiserror::Error;

use crate::{
    backend::{
        clients::brainy_backend_client::BrainyBackendClientError,
        dto::sign_up_request::SignUpRequest, models::UserInformationDto,
    },
    settings::services::settings_updater::SettingsUpdaterError,
};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum AuthenticatorError {
    #[error(transparent)]
    BrainyBackendClient(#[from] BrainyBackendClientError),
    #[error(transparent)]
    SettingsUpdater(#[from] SettingsUpdaterError),
}

#[async_trait]
pub trait Authenticator: Send + Sync {
    async fn sign_in(
        &self,
        username: String,
        password: String,
    ) -> Result<UserInformationDto, AuthenticatorError>;

    async fn sign_out(&self) -> Result<(), AuthenticatorError>;

    async fn sign_up(
        &self,
        request: SignUpRequest,
    ) -> Result<UserInformationDto, AuthenticatorError>;
}
