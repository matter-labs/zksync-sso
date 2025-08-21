use crate::config;
use sdk::api::{
    account::validators::is_module_validator as sdk_is_module_validator,
    utils::parse_address,
};

#[derive(Debug, uniffi::Record)]
pub struct IsModuleValidatorArgs {
    pub account: String,
    pub module_address: String,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum IsModuleValidatorError {
    #[error("{0}")]
    IsModuleValidator(String),
    #[error("Invalid address: {0}")]
    InvalidAccountAddress(String),
    #[error("Invalid module address: {0}")]
    InvalidModuleAddress(String),
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn is_module_validator(
    args: IsModuleValidatorArgs,
    config: config::Config,
) -> Result<bool, IsModuleValidatorError> {
    let account = parse_address(&args.account).map_err(|e| {
        IsModuleValidatorError::InvalidAccountAddress(e.to_string())
    })?;
    let module_address = parse_address(&args.module_address).map_err(|e| {
        IsModuleValidatorError::InvalidModuleAddress(e.to_string())
    })?;
    let sdk_config = config.try_into().map_err(|e: config::ConfigError| {
        IsModuleValidatorError::InvalidConfig(e.to_string())
    })?;
    sdk_is_module_validator(account, module_address, &sdk_config)
        .await
        .map_err(|e| IsModuleValidatorError::IsModuleValidator(e.to_string()))
}
