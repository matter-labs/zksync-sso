use MockPaymaster::MockPaymasterInstance;
use alloy::{
    primitives::{Address, U256},
    providers::Provider,
    sol,
};

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    MockPaymaster,
    "../../../../../../packages/erc4337-contracts/out/MockPaymaster.sol/MockPaymaster.json"
);

pub async fn deploy_mock_paymaster<P>(
    provider: P,
) -> eyre::Result<MockPaymasterInstance<P>>
where
    P: Provider + Send + Sync + Clone,
{
    let paymaster = MockPaymaster::deploy(provider.clone()).await?;
    Ok(paymaster)
}

pub async fn deposit_amount<P>(
    provider: P,
    address: Address,
    amount: U256,
) -> eyre::Result<()>
where
    P: Provider + Send + Sync + Clone,
{
    let paymaster = MockPaymaster::new(address, provider.clone());
    let mut deposit_request = paymaster.deposit().into_transaction_request();
    deposit_request.value = Some(amount);
    _ = provider.send_transaction(deposit_request).await?.get_receipt().await?;
    Ok(())
}

pub async fn deploy_mock_paymaster_and_deposit_amount<P>(
    amount: U256,
    provider: P,
) -> eyre::Result<(MockPaymasterInstance<P>, Address)>
where
    P: Provider + Send + Sync + Clone,
{
    let paymaster = deploy_mock_paymaster(provider.clone()).await?;
    let paymaster_address = paymaster.address().to_owned();
    deposit_amount(provider, paymaster_address, amount).await?;
    Ok((paymaster, paymaster_address))
}
