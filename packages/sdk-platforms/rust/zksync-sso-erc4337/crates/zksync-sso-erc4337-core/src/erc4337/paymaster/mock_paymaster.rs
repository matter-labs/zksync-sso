use crate::utils::alloy_utilities::test_utilities::node_backend::{
    TestNodeBackend, resolve_test_node_backend,
};
use MockPaymaster::MockPaymasterInstance;
use alloy::{
    consensus::Transaction as _,
    network::{ReceiptResponse as _, TransactionResponse as _},
    primitives::{Address, B256, U256},
    providers::{
        Network, PendingTransactionBuilder, PendingTransactionError, Provider,
        WalletProvider, WatchTxError,
    },
    rpc::types::{BlockId, BlockNumberOrTag, TransactionRequest},
    sol,
};
use log::{info, warn};
use std::time::Duration;
use tokio::time::sleep;

const TX_VISIBILITY_RETRIES: usize = 3;
const TX_VISIBILITY_DELAY: Duration = Duration::from_secs(1);
const TX_REBROADCAST_ATTEMPTS: usize = 1;

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
    P: Provider + WalletProvider + Send + Sync + Clone,
{
    match resolve_test_node_backend() {
        TestNodeBackend::ZkSyncOs => {
            deploy_mock_paymaster_zksyncos(provider).await
        }
        TestNodeBackend::Anvil => deploy_mock_paymaster_anvil(provider).await,
    }
}

pub async fn deposit_amount<P>(
    provider: P,
    address: Address,
    amount: U256,
) -> eyre::Result<()>
where
    P: Provider + WalletProvider + Send + Sync + Clone,
{
    match resolve_test_node_backend() {
        TestNodeBackend::ZkSyncOs => {
            deposit_amount_zksyncos(provider, address, amount).await
        }
        TestNodeBackend::Anvil => {
            deposit_amount_anvil(provider, address, amount).await
        }
    }
}

pub async fn deploy_mock_paymaster_and_deposit_amount<P>(
    amount: U256,
    provider: P,
) -> eyre::Result<(MockPaymasterInstance<P>, Address)>
where
    P: Provider + WalletProvider + Send + Sync + Clone,
{
    let paymaster = deploy_mock_paymaster(provider.clone()).await?;
    let paymaster_address = paymaster.address().to_owned();
    deposit_amount(provider, paymaster_address, amount).await?;
    Ok((paymaster, paymaster_address))
}

async fn deploy_mock_paymaster_anvil<P>(
    provider: P,
) -> eyre::Result<MockPaymasterInstance<P>>
where
    P: Provider + Send + Sync + Clone,
{
    let paymaster = MockPaymaster::deploy(provider.clone()).await?;
    Ok(paymaster)
}

async fn deploy_mock_paymaster_zksyncos<P>(
    provider: P,
) -> eyre::Result<MockPaymasterInstance<P>>
where
    P: Provider + WalletProvider + Send + Sync + Clone,
{
    let deploy_builder = MockPaymaster::deploy_builder(provider.clone());
    let from = provider.default_signer_address();
    let pending_nonce = provider
        .get_transaction_count(from)
        .block_id(BlockId::Number(BlockNumberOrTag::Pending))
        .await?;
    let pending = deploy_builder.nonce(pending_nonce).send().await?;
    let tx_hash = *pending.tx_hash();
    let tx = provider.get_transaction_by_hash(tx_hash).await?;
    if let Some(tx) = tx {
        let from = tx.from();
        let tx_nonce = tx.nonce();
        let latest_nonce = provider.get_transaction_count(from).await?;
        let pending_nonce = provider
            .get_transaction_count(from)
            .block_id(BlockId::Number(BlockNumberOrTag::Pending))
            .await?;
        eprintln!(
            "paymaster deploy tx sent: tx={tx_hash:?} from={from:?} tx_nonce={tx_nonce} latest_nonce={latest_nonce} pending_nonce={pending_nonce}"
        );
    } else {
        eprintln!("paymaster deploy tx sent: tx={tx_hash:?} (tx=None)");
    }
    let receipt = pending.get_receipt().await?;
    let contract_address = receipt
        .contract_address()
        .ok_or_else(|| eyre::eyre!("Paymaster contract not deployed"))?;
    eprintln!(
        "paymaster deploy confirmed: tx={tx_hash:?} block={:?} address={contract_address:?}",
        receipt.block_number
    );
    Ok(MockPaymaster::new(contract_address, provider))
}

async fn deposit_amount_anvil<P>(
    provider: P,
    address: Address,
    amount: U256,
) -> eyre::Result<()>
where
    P: Provider + WalletProvider + Send + Sync + Clone,
{
    let paymaster = MockPaymaster::new(address, provider.clone());
    let mut deposit_request = paymaster.deposit().into_transaction_request();
    deposit_request.value = Some(amount);
    _ = provider.send_transaction(deposit_request).await?.get_receipt().await?;
    Ok(())
}

async fn deposit_amount_zksyncos<P>(
    provider: P,
    address: Address,
    amount: U256,
) -> eyre::Result<()>
where
    P: Provider + WalletProvider + Send + Sync + Clone,
{
    let paymaster = MockPaymaster::new(address, provider.clone());
    info!("paymaster deposit request: address={address:?} amount={amount}");
    let mut deposit_request = paymaster.deposit().into_transaction_request();
    deposit_request.value = Some(amount);
    let from = provider.default_signer_address();
    let pending_nonce = provider
        .get_transaction_count(from)
        .block_id(BlockId::Number(BlockNumberOrTag::Pending))
        .await?;
    deposit_request.from = Some(from);
    deposit_request.nonce = Some(pending_nonce);
    info!("paymaster deposit using nonce: from={from:?} nonce={pending_nonce}");
    let pending = send_deposit_tx(&provider, &mut deposit_request).await?;
    let pending =
        ensure_tx_propagated(&provider, &mut deposit_request, pending).await?;
    let tx_hash = *pending.tx_hash();
    info!("paymaster deposit tx sent: {tx_hash:?}");
    let receipt = match pending
        .with_timeout(Some(Duration::from_secs(120)))
        .get_receipt()
        .await
    {
        Ok(receipt) => receipt,
        Err(PendingTransactionError::TxWatcher(WatchTxError::Timeout)) => {
            let tx = provider.get_transaction_by_hash(tx_hash).await?;
            if let Some(tx) = tx {
                let from = tx.from();
                let tx_nonce = tx.nonce();
                let latest_nonce = provider.get_transaction_count(from).await?;
                let pending_nonce = provider
                    .get_transaction_count(from)
                    .block_id(BlockId::Number(BlockNumberOrTag::Pending))
                    .await?;
                let balance = provider.get_balance(from).await?;
                let gas_price = provider.get_gas_price().await?;
                return Err(eyre::eyre!(
                    "paymaster deposit tx {tx_hash:?} timed out (from={from:?} tx_nonce={tx_nonce} latest_nonce={latest_nonce} pending_nonce={pending_nonce} balance={balance} gas_price={gas_price} tx={tx:?})"
                ));
            }
            return Err(eyre::eyre!(
                "paymaster deposit tx {tx_hash:?} timed out (tx=None)"
            ));
        }
        Err(err) => return Err(err.into()),
    };
    info!(
        "paymaster deposit confirmed: tx={tx_hash:?} block={:?}",
        receipt.block_number
    );
    Ok(())
}

async fn ensure_tx_propagated<P, N>(
    provider: &P,
    tx_request: &mut TransactionRequest,
    mut pending: PendingTransactionBuilder<N>,
) -> eyre::Result<PendingTransactionBuilder<N>>
where
    P: Provider<N> + WalletProvider + Send + Sync + Clone,
    N: Network<TransactionRequest = TransactionRequest>,
{
    for rebroadcast_attempt in 0..=TX_REBROADCAST_ATTEMPTS {
        let tx_hash = *pending.tx_hash();
        let mut missing_from_peers = false;
        for attempt in 0..TX_VISIBILITY_RETRIES {
            match tx_visibility(provider, tx_hash).await? {
                TxVisibility::Visible => return Ok(pending),
                TxVisibility::MissingNotFoundAmongPeers => {
                    missing_from_peers = true;
                    break;
                }
                TxVisibility::Missing => {
                    if attempt + 1 < TX_VISIBILITY_RETRIES {
                        sleep(TX_VISIBILITY_DELAY).await;
                    }
                }
            }
        }
        if rebroadcast_attempt == TX_REBROADCAST_ATTEMPTS {
            let reason = if missing_from_peers {
                "not found among peers"
            } else {
                "not found"
            };
            return Err(eyre::eyre!(
                "transaction {tx_hash:?} {reason} after {} checks; refusing to continue to avoid nonce gap",
                TX_VISIBILITY_RETRIES
            ));
        }
        if missing_from_peers {
            warn!(
                "transaction {tx_hash:?} not found among peers; rebroadcasting"
            );
        } else {
            warn!("transaction {tx_hash:?} not found; rebroadcasting");
        }
        pending = send_deposit_tx(provider, tx_request).await?;
    }
    unreachable!("rebroadcast loop should return or error");
}

enum TxVisibility {
    Visible,
    Missing,
    MissingNotFoundAmongPeers,
}

async fn tx_visibility<P, N>(
    provider: &P,
    tx_hash: B256,
) -> eyre::Result<TxVisibility>
where
    P: Provider<N> + Send + Sync,
    N: Network,
{
    match provider.get_transaction_by_hash(tx_hash).await {
        Ok(Some(_)) => Ok(TxVisibility::Visible),
        Ok(None) => Ok(TxVisibility::Missing),
        Err(err) => {
            let Some(error_resp) = err.as_error_resp() else {
                return Err(err.into());
            };
            let message = error_resp.message.to_lowercase();
            if message.contains("not found among peers") {
                return Ok(TxVisibility::MissingNotFoundAmongPeers);
            }
            Err(err.into())
        }
    }
}

async fn send_deposit_tx<P, N>(
    provider: &P,
    tx_request: &mut TransactionRequest,
) -> eyre::Result<PendingTransactionBuilder<N>>
where
    P: Provider<N> + WalletProvider + Send + Sync + Clone,
    N: Network<TransactionRequest = TransactionRequest>,
{
    match provider.send_transaction(tx_request.clone()).await {
        Ok(pending) => Ok(pending),
        Err(err) => {
            let Some(error_resp) = err.as_error_resp() else {
                return Err(err.into());
            };
            if !error_resp.message.contains("execution reverted") {
                return Err(err.into());
            }
            warn!(
                "paymaster deposit gas estimation failed (code {}): {}; retrying with manual gas limit",
                error_resp.code, error_resp.message
            );
            tx_request.gas = Some(1_000_000u64);
            Ok(provider.send_transaction(tx_request.clone()).await?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::alloy_utilities::test_utilities::{
        config::TestInfraConfig,
        start_node_and_deploy_contracts_and_start_bundler_with_config,
    };
    use alloy::primitives::U256;

    #[tokio::test]
    async fn test_deploy_mock_paymaster_and_deposit_amount() -> eyre::Result<()>
    {
        let (
            _,
            node_handle,
            provider,
            _contracts,
            _signer_private_key,
            bundler,
            _bundler_client,
        ) = start_node_and_deploy_contracts_and_start_bundler_with_config(
            &TestInfraConfig::rich_wallet_9(),
        )
        .await?;

        let paymaster_fund_amount = U256::from(1_000_000_000_000_000_000u64);
        let (_mock_paymaster, paymaster_address) =
            deploy_mock_paymaster_and_deposit_amount(
                paymaster_fund_amount,
                provider.clone(),
            )
            .await?;
        eyre::ensure!(
            paymaster_address != Address::ZERO,
            "paymaster address is zero"
        );

        drop(bundler);
        drop(node_handle);
        Ok(())
    }
}
