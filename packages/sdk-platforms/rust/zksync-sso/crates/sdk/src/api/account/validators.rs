use crate::{
    client::modular_account::is_module_validator as is_module_validator_client,
    config::Config,
};
use alloy::primitives::Address;

pub async fn is_module_validator(
    account_address: Address,
    module_address: Address,
    config: &Config,
) -> eyre::Result<bool> {
    is_module_validator_client(account_address, module_address, config).await
}
