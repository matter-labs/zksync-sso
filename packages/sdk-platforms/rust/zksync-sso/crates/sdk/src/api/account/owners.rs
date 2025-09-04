use crate::{
    client::modular_account::is_k1_owner as is_k1_owner_client, config::Config,
};
use alloy::primitives::Address;

#[allow(dead_code)]
pub async fn is_k1_owner(
    account_address: Address,
    owner_address: Address,
    config: &Config,
) -> eyre::Result<bool> {
    is_k1_owner_client(account_address, owner_address, config).await
}
