use crate::{
    client::session::actions::session::{
        create::create_session as client_create_session, send::SignFn,
    },
    config::Config,
    utils::session::session_lib::session_spec::SessionSpec,
};
use alloy::{primitives::Address, signers::local::PrivateKeySigner};
use alloy_zksync::{
    network::unsigned_tx::eip712::PaymasterParams, wallet::ZksyncWallet,
};
use std::str::FromStr;

/// Arguments for creating a new session
#[derive(Debug, Clone)]
pub struct CreateSessionArgs {
    /// Account address
    pub account: Address,
    /// Configuration for the session
    pub session_config: SessionSpec,
    /// Optional paymaster configuration
    pub paymaster: Option<PaymasterParams>,
}

/// Return type for session creation
#[derive(Debug, Clone)]
pub struct CreateSessionReturnType {
    /// Transaction receipt after session creation
    pub transaction_receipt_json: String,
}

impl From<CreateSessionArgs>
    for crate::client::session::actions::session::create::CreateSessionArgs
{
    fn from(args: CreateSessionArgs) -> Self {
        crate::client::session::actions::session::create::CreateSessionArgs {
            account: args.account,
            session_config: args.session_config,
            paymaster: args.paymaster,
        }
    }
}

pub async fn create_session(
    args: CreateSessionArgs,
    sign_fn: SignFn,
    config: &Config,
) -> eyre::Result<CreateSessionReturnType> {
    let result = client_create_session(args.into(), sign_fn, config).await?;
    let transaction_receipt_json =
        serde_json::to_string(&result.transaction_receipt).map_err(|e| {
            eyre::eyre!("Failed to serialize transaction receipt: {}", e)
        })?;
    Ok(CreateSessionReturnType { transaction_receipt_json })
}

pub fn private_key_to_address(private_key_hex: &str) -> eyre::Result<Address> {
    let signer = PrivateKeySigner::from_str(private_key_hex)?;
    let wallet = ZksyncWallet::from(signer);
    let address = wallet.default_signer().address();
    Ok(address)
}
