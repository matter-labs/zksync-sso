use crate::erc4337::account::modular_smart_account::session::SessionKeyValidator;
use alloy::{
    primitives::{Address, FixedBytes},
    providers::Provider,
};

pub async fn get_session_status<P: Provider + Send + Sync + Clone>(
    account_address: Address,
    session_hash: FixedBytes<32>,
    session_key_validator_address: Address,
    provider: P,
) -> eyre::Result<u8> {
    let session_key_validator =
        SessionKeyValidator::new(session_key_validator_address, provider);
    let status = session_key_validator
        .sessionStatus(account_address, session_hash)
        .call()
        .await?;
    Ok(status)
}
