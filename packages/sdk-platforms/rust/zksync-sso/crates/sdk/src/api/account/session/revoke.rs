use crate::{
    client::session::actions::session::{
        revoke::revoke_session as client_revoke_session, send::SignFn,
    },
    config::Config,
};
use alloy::{
    hex::{FromHex, FromHexError},
    primitives::{Address, FixedBytes},
    signers::local::PrivateKeySigner,
};
use alloy_zksync::wallet::ZksyncWallet;
use std::str::FromStr;

/// Arguments for revoking a session
#[derive(Debug, Clone)]
pub struct RevokeSessionArgs {
    /// Session ID hex string
    pub session_id: String,
}

/// Return type for session revocation
#[derive(Debug, Clone)]
pub struct RevokeSessionReturnType {
    /// Transaction receipt after session revocation
    pub transaction_receipt_json: String,
}

impl TryFrom<RevokeSessionArgs>
    for crate::client::session::actions::session::revoke::RevokeSessionArgs
{
    type Error = FromHexError;

    fn try_from(args: RevokeSessionArgs) -> Result<Self, Self::Error> {
        let session_id_bytes = FixedBytes::<32>::from_hex(args.session_id)?;
        Ok(crate::client::session::actions::session::revoke::RevokeSessionArgs {
            session_id: session_id_bytes.into(),
        })
    }
}

pub async fn revoke_session(
    args: RevokeSessionArgs,
    account_address: Address,
    sign_fn: SignFn,
    config: &Config,
) -> eyre::Result<RevokeSessionReturnType> {
    let args = args.try_into()?;
    let result =
        client_revoke_session(args, account_address, sign_fn, config).await?;
    let transaction_receipt_json =
        serde_json::to_string(&result.transaction_receipt).map_err(|e| {
            eyre::eyre!("Failed to serialize transaction receipt: {}", e)
        })?;
    Ok(RevokeSessionReturnType { transaction_receipt_json })
}

pub fn private_key_to_address(private_key_hex: &str) -> eyre::Result<Address> {
    let signer = PrivateKeySigner::from_str(private_key_hex)?;
    let wallet = ZksyncWallet::from(signer);
    let address = wallet.default_signer().address();
    Ok(address)
}
