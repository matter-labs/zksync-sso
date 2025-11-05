use alloy::{
    primitives::{Address, U256},
    providers::Provider,
    rpc::types::TransactionRequest,
};

pub async fn fund_account<P>(
    account_address: Address,
    amount: U256,
    provider: P,
) -> eyre::Result<()>
where
    P: Provider + Send + Sync + Clone,
{
    let fund_tx =
        TransactionRequest::default().to(account_address).value(amount);
    _ = provider.send_transaction(fund_tx).await?.get_receipt().await?;
    Ok(())
}

pub async fn fund_account_with_default_amount<P>(
    account_address: Address,
    provider: P,
) -> eyre::Result<()>
where
    P: Provider + Send + Sync + Clone,
{
    fund_account(account_address, U256::from(1000000000000000000u64), provider)
        .await
}
