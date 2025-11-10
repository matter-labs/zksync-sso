use crate::erc4337::account::modular_smart_account::guardian::contract::GuardianExecutor;
use alloy::{primitives::Address, providers::Provider};

pub async fn get_guardians_list<P>(
    account_address: Address,
    guardian_executor_address: Address,
    provider: P,
) -> eyre::Result<Vec<Address>>
where
    P: Provider + Send + Sync + Clone,
{
    let guardian_executor =
        GuardianExecutor::new(guardian_executor_address, provider);
    let guardians =
        guardian_executor.guardiansFor(account_address).call().await?;
    Ok(guardians)
}
