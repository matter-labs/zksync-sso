use crate::erc4337::account::account_base::AccountBase;
use alloy::{primitives::Address, providers::Provider};

pub async fn get_entry_point_address<P: Provider + Send + Sync + Clone>(
    account_address: Address,
    provider: P,
) -> eyre::Result<Address> {
    let account_instance = AccountBase::new(account_address, provider.clone());
    let entry_point_address = account_instance.ENTRY_POINT_V08().call().await?;
    Ok(entry_point_address)
}
