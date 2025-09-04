use crate::config;
use sdk::api::{
    account::session::{
        create::CreateSessionArgs as SdkCreateSessionArgs,
        decode_session_config,
    },
    utils::{
        parse_address, parse_paymaster_params, sign_fn_from_private_key_hex,
    },
};

#[derive(Debug, uniffi::Record)]
pub struct CreateSessionArgs {
    pub account: String,
    pub session_config: String,
    pub owner_private_key: String,
    pub paymaster: Option<String>,
}

#[derive(Debug, uniffi::Record)]
pub struct CreateSessionReturnType {
    pub transaction_receipt_json: String,
}

impl From<CreateSessionReturnType>
    for sdk::api::account::session::create::CreateSessionReturnType
{
    fn from(create_session_return_type: CreateSessionReturnType) -> Self {
        sdk::api::account::session::create::CreateSessionReturnType {
            transaction_receipt_json: create_session_return_type
                .transaction_receipt_json,
        }
    }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum CreateSessionError {
    #[error("{0}")]
    CreateSession(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Invalid session config: {0}")]
    InvalidSessionConfig(String),
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    #[error("Invalid paymaster: {0}")]
    InvalidPaymaster(String),
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn create_session(
    args: CreateSessionArgs,
    config: config::Config,
) -> Result<CreateSessionReturnType, CreateSessionError> {
    let session_config = decode_session_config(&args.session_config)
        .map_err(|e| CreateSessionError::InvalidSessionConfig(e.to_string()))?;
    let account = parse_address(&args.account)
        .map_err(|e| CreateSessionError::InvalidAddress(e.to_string()))?;
    let owner_private_key = &args.owner_private_key;
    let paymaster = args
        .paymaster
        .map(|paymaster| {
            let input: Option<String> = None;
            parse_paymaster_params(paymaster, input)
        })
        .transpose()
        .map_err(|e| CreateSessionError::InvalidPaymaster(e.to_string()))?;
    let args = SdkCreateSessionArgs { account, session_config, paymaster };

    let sign_fn = sign_fn_from_private_key_hex(owner_private_key)
        .map_err(|e| CreateSessionError::InvalidPrivateKey(e.to_string()))?;

    let sdk_config: sdk::config::Config =
        config.try_into().map_err(|e: config::ConfigError| {
            CreateSessionError::InvalidConfig(e.to_string())
        })?;

    let result = sdk::api::account::session::create::create_session(
        args,
        sign_fn,
        &sdk_config,
    )
    .await
    .map_err(|e| CreateSessionError::CreateSession(e.to_string()))?;
    Ok(CreateSessionReturnType {
        transaction_receipt_json: result.transaction_receipt_json,
    })
}
