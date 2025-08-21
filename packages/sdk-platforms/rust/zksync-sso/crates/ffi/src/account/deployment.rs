use crate::{account::passkey::passkey_parameters::PasskeyParameters, config};
use sdk::api::{
    account::{
        deployment::{
            deploy_account as sdk_deploy_account,
            deploy_account_with_unique_id as sdk_deploy_account_with_unique_id,
        },
        session::decode_session_config,
    },
    utils::parse_address,
};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum DeployAccountError {
    #[error("{0}")]
    Deploy(String),

    #[error("Account already deployed")]
    AccountAlreadyDeployed,

    #[error("Invalid session config: {0}")]
    InvalidSessionConfig(String),

    #[error("Invalid K1 owners: {0}")]
    InvalidK1Owners(String),
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn deploy_account(
    passkey_parameters: PasskeyParameters,
    initial_k1_owners: Option<Vec<String>>,
    initial_session_config_json: Option<String>,
    config: config::Config,
) -> Result<super::Account, DeployAccountError> {
    let initial_k1_owners = initial_k1_owners
        .map(|k1_owners| {
            k1_owners
                .into_iter()
                .map(|k1_owner| parse_address(&k1_owner))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()
        .map_err(|e| DeployAccountError::InvalidK1Owners(e.to_string()))?;

    let initial_session = initial_session_config_json
        .map(|session_config| {
            decode_session_config(&session_config).map_err(|e| {
                DeployAccountError::InvalidSessionConfig(e.to_string())
            })
        })
        .transpose()
        .map_err(|e| DeployAccountError::InvalidSessionConfig(e.to_string()))?;

    let sdk_config = config.try_into().map_err(|e: config::ConfigError| {
        DeployAccountError::Deploy(e.to_string())
    })?;

    sdk_deploy_account(
        passkey_parameters.into(),
        initial_k1_owners,
        initial_session,
        &sdk_config,
    )
    .await
    .map_err(|e| DeployAccountError::Deploy(e.to_string()))
    .map(Into::into)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn deploy_account_with_unique_id(
    passkey_parameters: PasskeyParameters,
    unique_account_id: String,
    initial_k1_owners: Option<Vec<String>>,
    initial_session_config_json: Option<String>,
    config: config::Config,
) -> Result<super::Account, DeployAccountError> {
    let initial_k1_owners = initial_k1_owners
        .map(|k1_owners| {
            k1_owners
                .into_iter()
                .map(|k1_owner| parse_address(&k1_owner))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()
        .map_err(|e| DeployAccountError::InvalidK1Owners(e.to_string()))?;

    let initial_session = initial_session_config_json
        .map(|session_config| {
            decode_session_config(&session_config).map_err(|e| {
                DeployAccountError::InvalidSessionConfig(e.to_string())
            })
        })
        .transpose()
        .map_err(|e| DeployAccountError::InvalidSessionConfig(e.to_string()))?;

    let sdk_config = config.try_into().map_err(|e: config::ConfigError| {
        DeployAccountError::Deploy(e.to_string())
    })?;

    sdk_deploy_account_with_unique_id(
        passkey_parameters.into(),
        unique_account_id,
        initial_k1_owners,
        initial_session,
        &sdk_config,
    )
    .await
    .map_err(|e| DeployAccountError::Deploy(e.to_string()))
    .map(Into::into)
}
