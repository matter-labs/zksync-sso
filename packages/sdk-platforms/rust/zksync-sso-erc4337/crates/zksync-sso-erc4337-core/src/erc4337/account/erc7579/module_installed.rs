use crate::erc4337::account::erc7579::IERC7579Account;
use alloy::{
    primitives::{Address, Bytes, U256},
    providers::Provider,
};
use eyre::{self, Ok};

pub async fn is_module_installed<P: Provider + Send + Sync + Clone>(
    module: Address,
    account: Address,
    provider: P,
) -> eyre::Result<bool> {
    let account = IERC7579Account::new(account, provider.clone());
    let module_type_id = U256::from(1);
    let is_installed = account
        .isModuleInstalled(module_type_id, module, Bytes::default())
        .call()
        .await?;
    Ok(is_installed)
}
