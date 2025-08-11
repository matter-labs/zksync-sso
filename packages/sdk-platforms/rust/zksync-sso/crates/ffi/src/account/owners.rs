use crate::config;
use sdk::api::{
    account::owners::is_k1_owner as is_k1_owner_sdk, utils::parse_address,
};

#[derive(Debug, uniffi::Record)]
pub struct IsK1OwnerArgs {
    pub account: String,
    pub owner_address: String,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum IsK1OwnerError {
    #[error("{0}")]
    IsK1Owner(String),
    #[error("Invalid address: {0}")]
    InvalidAccountAddress(String),
    #[error("Invalid owner address: {0}")]
    InvalidOwnerAddress(String),
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn is_k1_owner(
    args: IsK1OwnerArgs,
    config: config::Config,
) -> Result<bool, IsK1OwnerError> {
    let account = parse_address(&args.account)
        .map_err(|e| IsK1OwnerError::InvalidAccountAddress(e.to_string()))?;
    let owner_address = parse_address(&args.owner_address)
        .map_err(|e| IsK1OwnerError::InvalidOwnerAddress(e.to_string()))?;
    let result = is_k1_owner_sdk(
        account,
        owner_address,
        &(config.try_into()
            as Result<sdk::config::Config, config::ConfigError>)
            .map_err(|e: config::ConfigError| {
                IsK1OwnerError::InvalidConfig(e.to_string())
            })?,
    )
    .await
    .map_err(|e| IsK1OwnerError::IsK1Owner(e.to_string()))?;
    Ok(result)
}
