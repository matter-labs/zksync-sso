use crate::config;

#[derive(Debug, uniffi::Record)]
pub struct AndroidRpId {
    pub origin: String,
    pub rp_id: String,
}

impl From<AndroidRpId> for sdk::api::account::deployment::AndroidRpId {
    fn from(android_rp_id: AndroidRpId) -> Self {
        sdk::api::account::deployment::AndroidRpId {
            origin: android_rp_id.origin,
            rp_id: android_rp_id.rp_id,
        }
    }
}

#[derive(Debug, uniffi::Enum)]
pub enum RpId {
    Apple(String),
    Android(AndroidRpId),
}

impl RpId {
    pub fn new_apple(rp_id: String) -> Self {
        Self::Apple(rp_id)
    }

    pub fn new_android(origin: String, rp_id: String) -> Self {
        Self::Android(AndroidRpId { origin, rp_id })
    }

    pub fn origin(&self) -> &str {
        match self {
            RpId::Apple(rp_id) => rp_id,
            RpId::Android(android_rp_id) => &android_rp_id.origin,
        }
    }

    pub fn identifier(&self) -> &str {
        match self {
            RpId::Apple(rp_id) => rp_id,
            RpId::Android(android_rp_id) => &android_rp_id.rp_id,
        }
    }
}

impl From<RpId> for sdk::api::account::deployment::RpId {
    fn from(rp_id: RpId) -> Self {
        match rp_id {
            RpId::Apple(rp_id) => {
                sdk::api::account::deployment::RpId::Apple(rp_id)
            }
            RpId::Android(android_rp_id) => {
                sdk::api::account::deployment::RpId::Android(
                    android_rp_id.into(),
                )
            }
        }
    }
}

#[derive(Debug, uniffi::Record)]
pub struct PasskeyParameters {
    pub credential_raw_attestation_object: Vec<u8>,
    pub credential_raw_client_data_json: Vec<u8>,
    pub credential_id: Vec<u8>,
    pub rp_id: RpId,
}

impl From<PasskeyParameters>
    for sdk::api::account::deployment::PasskeyParameters
{
    fn from(passkey_parameters: PasskeyParameters) -> Self {
        sdk::api::account::deployment::PasskeyParameters {
            credential_raw_attestation_object: passkey_parameters
                .credential_raw_attestation_object,
            credential_raw_client_data_json: passkey_parameters
                .credential_raw_client_data_json,
            credential_id: passkey_parameters.credential_id,
            rp_id: passkey_parameters.rp_id.into(),
        }
    }
}

impl From<sdk::api::account::deployment::DeployedAccountDetails>
    for super::Account
{
    fn from(
        deployed_account_details: sdk::api::account::deployment::DeployedAccountDetails,
    ) -> Self {
        Self {
            address: deployed_account_details.address.to_string(),
            unique_account_id: deployed_account_details.unique_account_id,
        }
    }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum DeployAccountError {
    #[error("{0}")]
    Deploy(String),

    #[error("Account already deployed")]
    AccountAlreadyDeployed,
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn deploy_account(
    passkey_parameters: PasskeyParameters,
    config: config::Config,
) -> Result<super::Account, DeployAccountError> {
    sdk::api::account::deployment::deploy_account(
        passkey_parameters.into(),
        &(config.try_into()
            as Result<sdk::config::Config, config::ConfigError>)
            .map_err(|e: config::ConfigError| {
                DeployAccountError::Deploy(e.to_string())
            })?,
    )
    .await
    .map_err(|e| DeployAccountError::Deploy(e.to_string()))
    .map(Into::into)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn deploy_account_with_unique_id(
    passkey_parameters: PasskeyParameters,
    unique_account_id: String,
    config: config::Config,
) -> Result<super::Account, DeployAccountError> {
    sdk::api::account::deployment::deploy_account_with_unique_id(
        passkey_parameters.into(),
        unique_account_id,
        &(config.try_into()
            as Result<sdk::config::Config, config::ConfigError>)
            .map_err(|e: config::ConfigError| {
                DeployAccountError::Deploy(e.to_string())
            })?,
    )
    .await
    .map_err(|e| DeployAccountError::Deploy(e.to_string()))
    .map(Into::into)
}
