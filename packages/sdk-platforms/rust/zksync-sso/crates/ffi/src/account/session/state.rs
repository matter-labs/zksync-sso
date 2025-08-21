use crate::config;
use sdk::api::{
    account::session::{
        decode_session_config,
        state::{
            GetSessionStateArgs as SdkGetSessionStateArgs,
            get_session_state as sdk_get_session_state,
        },
    },
    utils::parse_address,
};

#[derive(Debug, uniffi::Record)]
pub struct GetSessionStateArgs {
    pub account: String,
    pub session_config: String,
}

#[derive(Debug, uniffi::Record)]
pub struct GetSessionStateReturnType {
    pub session_state_json: String,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum GetSessionStateError {
    #[error("{0}")]
    GetSessionState(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Invalid session config: {0}")]
    InvalidSessionConfig(String),
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn get_session_state(
    args: GetSessionStateArgs,
    config: config::Config,
) -> Result<GetSessionStateReturnType, GetSessionStateError> {
    let session_config =
        decode_session_config(&args.session_config).map_err(|e| {
            GetSessionStateError::InvalidSessionConfig(e.to_string())
        })?;
    let account = parse_address(&args.account)
        .map_err(|e| GetSessionStateError::InvalidAddress(e.to_string()))?;

    let sdk_args = SdkGetSessionStateArgs { account, session_config };

    let sdk_config = config.try_into().map_err(|e: config::ConfigError| {
        GetSessionStateError::GetSessionState(e.to_string())
    })?;

    let result = sdk_get_session_state(sdk_args, &sdk_config)
        .await
        .map_err(|e| GetSessionStateError::GetSessionState(e.to_string()))?;

    let session_state_json = serde_json::to_string(&result.session_state)
        .map_err(|e| GetSessionStateError::GetSessionState(e.to_string()))?;

    Ok(GetSessionStateReturnType { session_state_json })
}
