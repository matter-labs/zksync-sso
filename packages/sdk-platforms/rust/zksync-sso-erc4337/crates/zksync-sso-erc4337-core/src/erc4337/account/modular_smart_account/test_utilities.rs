use crate::utils::alloy_utilities::test_utilities::node_backend::{
    TestNodeBackend, resolve_test_node_backend,
};
use alloy::{
    consensus::Transaction as _,
    network::TransactionResponse as _,
    primitives::{Address, B256, U256},
    providers::{
        Network, PendingTransactionBuilder, PendingTransactionError, Provider,
        WalletProvider, WatchTxError,
    },
    rpc::types::{BlockId, BlockNumberOrTag, TransactionRequest},
};
use log::{info, warn};
use std::time::Duration;
use tokio::time::sleep;

const TX_VISIBILITY_RETRIES: usize = 3;
const TX_VISIBILITY_DELAY: Duration = Duration::from_secs(1);
const TX_REBROADCAST_ATTEMPTS: usize = 1;

pub async fn fund_account<P>(
    account_address: Address,
    amount: U256,
    provider: P,
) -> eyre::Result<()>
where
    P: Provider + WalletProvider + Send + Sync + Clone,
{
    match resolve_test_node_backend() {
        TestNodeBackend::ZkSyncOs => {
            fund_account_zksyncos(account_address, amount, provider).await
        }
        TestNodeBackend::Anvil => {
            fund_account_anvil(account_address, amount, provider).await
        }
    }
}

pub async fn fund_account_with_default_amount<P>(
    account_address: Address,
    provider: P,
) -> eyre::Result<()>
where
    P: Provider + WalletProvider + Send + Sync + Clone,
{
    fund_account(account_address, U256::from(1000000000000000000u64), provider)
        .await
}

async fn fund_account_anvil<P>(
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

async fn fund_account_zksyncos<P>(
    account_address: Address,
    amount: U256,
    provider: P,
) -> eyre::Result<()>
where
    P: Provider + WalletProvider + Send + Sync + Clone,
{
    let mut fund_tx =
        TransactionRequest::default().to(account_address).value(amount);
    let from = provider.default_signer_address();
    let pending_nonce = provider
        .get_transaction_count(from)
        .block_id(BlockId::Number(BlockNumberOrTag::Pending))
        .await?;
    fund_tx.from = Some(from);
    fund_tx.nonce = Some(pending_nonce);
    info!(
        "fund account using nonce: to={account_address:?} from={from:?} nonce={pending_nonce} amount={amount}"
    );
    let pending = send_fund_tx(&provider, &mut fund_tx).await?;
    let pending =
        ensure_tx_propagated(&provider, &mut fund_tx, pending).await?;
    let tx_hash = *pending.tx_hash();
    info!("fund account tx sent: {tx_hash:?}");
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
                    "fund account tx {tx_hash:?} timed out (from={from:?} tx_nonce={tx_nonce} latest_nonce={latest_nonce} pending_nonce={pending_nonce} balance={balance} gas_price={gas_price} tx={tx:?})"
                ));
            }
            return Err(eyre::eyre!(
                "fund account tx {tx_hash:?} timed out (tx=None)"
            ));
        }
        Err(err) => return Err(err.into()),
    };
    info!(
        "fund account confirmed: tx={tx_hash:?} block={:?}",
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
        pending = send_fund_tx(provider, tx_request).await?;
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

async fn send_fund_tx<P, N>(
    provider: &P,
    tx_request: &mut TransactionRequest,
) -> eyre::Result<PendingTransactionBuilder<N>>
where
    P: Provider<N> + WalletProvider + Send + Sync + Clone,
    N: Network<TransactionRequest = TransactionRequest>,
{
    Ok(provider.send_transaction(tx_request.clone()).await?)
}
