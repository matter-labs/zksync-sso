use crate::config;
use sdk::api::{
    account::session::revoke::RevokeSessionArgs as SdkRevokeSessionArgs,
    utils::{parse_address, sign_fn_from_private_key_hex},
};

#[derive(Debug, uniffi::Record)]
pub struct RevokeSessionArgs {
    pub account: String,
    pub session_id: String,
    pub owner_private_key: String,
}

#[derive(Debug, uniffi::Record)]
pub struct RevokeSessionReturnType {
    pub transaction_receipt_json: String,
}

impl From<RevokeSessionReturnType>
    for sdk::api::account::session::revoke::RevokeSessionReturnType
{
    fn from(revoke_session_return_type: RevokeSessionReturnType) -> Self {
        sdk::api::account::session::revoke::RevokeSessionReturnType {
            transaction_receipt_json: revoke_session_return_type
                .transaction_receipt_json,
        }
    }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum RevokeSessionError {
    #[error("{0}")]
    RevokeSession(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn revoke_session(
    args: RevokeSessionArgs,
    config: config::Config,
) -> Result<RevokeSessionReturnType, RevokeSessionError> {
    let account_address = parse_address(&args.account)
        .map_err(|e| RevokeSessionError::InvalidAddress(e.to_string()))?;

    let sign_fn = sign_fn_from_private_key_hex(&args.owner_private_key)
        .map_err(|e| RevokeSessionError::RevokeSession(e.to_string()))?;

    let sdk_args = SdkRevokeSessionArgs { session_id: args.session_id };

    let result = sdk::api::account::session::revoke::revoke_session(
        sdk_args,
        account_address,
        sign_fn,
        &(config.try_into()
            as Result<sdk::config::Config, config::ConfigError>)
            .map_err(|e: config::ConfigError| {
                RevokeSessionError::RevokeSession(e.to_string())
            })?,
    )
    .await
    .map_err(|e| RevokeSessionError::RevokeSession(e.to_string()))?;

    Ok(RevokeSessionReturnType {
        transaction_receipt_json: result.transaction_receipt_json,
    })
}
