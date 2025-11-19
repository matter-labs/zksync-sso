use crate::erc4337::account::erc7579::{
    contract::account::IERC7579Account, module::Module,
};
use alloy::{
    primitives::{Address, Bytes},
    providers::Provider,
};

#[derive(Clone)]
pub struct IsModuleInstalledParams<P: Provider + Send + Sync + Clone> {
    pub module: Module,
    pub account: Address,
    pub provider: P,
}

pub async fn is_module_installed<P>(
    params: IsModuleInstalledParams<P>,
) -> eyre::Result<bool>
where
    P: Provider + Send + Sync + Clone,
{
    let IsModuleInstalledParams { module, account, provider } = params;
    let account = IERC7579Account::new(account, provider.clone());
    let module_type = module.module_type;
    let module_address = module.address;
    let is_installed = account
        .isModuleInstalled(module_type.into(), module_address, Bytes::default())
        .call()
        .await?;
    Ok(is_installed)
}
