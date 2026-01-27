#[cfg(test)]
use crate::utils::alloy_utilities::test_utilities::node_backend::{
    TestNodeBackend, resolve_test_node_backend,
};
#[cfg(test)]
use alloy::providers::ProviderBuilder;
#[cfg(test)]
use alloy::{eips::BlockId, primitives::U256, rpc::types::TransactionRequest};
use alloy::{
    primitives::Address,
    providers::{Provider, WalletProvider},
    rpc::types::{BlockNumberOrTag, Filter, FilterSet, Log},
    sol_types::SolEvent,
};
use std::collections::HashSet;
#[cfg(test)]
use tokio::time::{Duration, sleep};
#[cfg(test)]
use url::Url;

#[cfg(test)]
const DEFAULT_L1_RPC_URL: &str = "http://127.0.0.1:8545";

pub const MAX_BLOCK_RANGE: u64 = 100_000;

pub fn calculate_from_block(current_block: u64) -> u64 {
    current_block.saturating_sub(MAX_BLOCK_RANGE)
}

pub fn create_logs_filter_with_range(
    contract_address: Address,
    from_block: u64,
) -> Filter {
    Filter {
        address: contract_address.into(),
        topics: [
            FilterSet::default(),
            FilterSet::default(),
            FilterSet::default(),
            FilterSet::default(),
        ],
        block_option: alloy::rpc::types::FilterBlockOption::Range {
            from_block: Some(BlockNumberOrTag::Number(from_block)),
            to_block: None,
        },
    }
}

pub fn parse_add_remove_events<AddItem, RemoveKey, AddEvent, RemoveEvent>(
    logs: Vec<Log>,
    account_address: Address,
    add_topic: alloy::primitives::FixedBytes<32>,
    remove_topic: alloy::primitives::FixedBytes<32>,
    extract_add: impl Fn(AddEvent, Address) -> Option<AddItem>,
    extract_remove: impl Fn(RemoveEvent, Address) -> Option<RemoveKey>,
) -> (Vec<AddItem>, HashSet<RemoveKey>)
where
    AddEvent: SolEvent,
    RemoveEvent: SolEvent,
    RemoveKey: std::hash::Hash + Eq,
{
    let mut added_items: Vec<AddItem> = Vec::new();
    let mut removed_keys: HashSet<RemoveKey> = HashSet::new();

    for log in logs {
        if let Some(topic0) = log.inner.topics().first() {
            if *topic0 == add_topic {
                if let Ok(decoded) = log.log_decode::<AddEvent>() {
                    let event = decoded.inner.data;
                    if let Some(item) = extract_add(event, account_address) {
                        added_items.push(item);
                    }
                }
            } else if *topic0 == remove_topic
                && let Ok(decoded) = log.log_decode::<RemoveEvent>()
            {
                let event = decoded.inner.data;
                if let Some(key) = extract_remove(event, account_address) {
                    removed_keys.insert(key);
                }
            }
        }
    }

    (added_items, removed_keys)
}

/// Advance the EVM time by the specified number of seconds.
/// This is useful for testing time-dependent functionality like recovery delays.
///
/// # Arguments
/// * `provider` - The provider connected to an Anvil instance
/// * `seconds` - Number of seconds to advance time by
///
/// # Returns
/// The new block number after mining (as a hex string)
pub async fn advance_time<P>(provider: &P, seconds: u64) -> eyre::Result<String>
where
    P: Provider + WalletProvider + Send + Sync + Clone,
{
    #[cfg(test)]
    match resolve_test_node_backend() {
        TestNodeBackend::Anvil => {
            // Increase time by the specified number of seconds
            let _: u64 = provider
                .raw_request("evm_increaseTime".into(), (seconds,))
                .await?;

            // Mine a block to apply the time change
            // evm_mine returns the new block number as a hex string
            let block_number: String =
                provider.raw_request("evm_mine".into(), ()).await?;

            Ok(block_number)
        }
        TestNodeBackend::ZkSyncOs => {
            let start_ts = latest_block_timestamp(provider).await?;
            let target_ts = start_ts.saturating_add(seconds).saturating_add(1);

            let l1_url = std::env::var("SSO_ZKSYNC_OS_L1_RPC_URL")
                .unwrap_or_else(|_| DEFAULT_L1_RPC_URL.to_string());
            let l1_url = Url::parse(&l1_url)?;
            let l1_provider = ProviderBuilder::new().connect_http(l1_url);
            for _ in 0..20 {
                let l1_ts = latest_block_timestamp(&l1_provider).await?;
                if target_ts > l1_ts {
                    let delta = target_ts - l1_ts;
                    let _: u64 = l1_provider
                        .raw_request("evm_increaseTime".into(), (delta,))
                        .await?;
                    let _: String =
                        l1_provider.raw_request("evm_mine".into(), ()).await?;
                }

                let from = provider.default_signer_address();
                let pending_nonce = provider
                    .get_transaction_count(from)
                    .block_id(BlockId::Number(BlockNumberOrTag::Pending))
                    .await?;
                let tx = TransactionRequest::default()
                    .from(from)
                    .to(from)
                    .nonce(pending_nonce)
                    .value(U256::ZERO);
                _ = provider.send_transaction(tx).await?.get_receipt().await?;

                let latest_ts = latest_block_timestamp(provider).await?;
                if latest_ts >= target_ts {
                    let block_number: String = provider
                        .raw_request("eth_blockNumber".into(), ())
                        .await?;
                    return Ok(block_number);
                }
                sleep(Duration::from_millis(250)).await;
            }

            let block_number: String =
                provider.raw_request("eth_blockNumber".into(), ()).await?;
            Ok(block_number)
        }
    }

    #[cfg(not(test))]
    {
        let _: u64 =
            provider.raw_request("evm_increaseTime".into(), (seconds,)).await?;
        let block_number: String =
            provider.raw_request("evm_mine".into(), ()).await?;
        Ok(block_number)
    }
}

#[cfg(test)]
pub(crate) async fn latest_block_timestamp<P>(provider: &P) -> eyre::Result<u64>
where
    P: Provider + Send + Sync + Clone,
{
    let block = provider
        .get_block_by_number(BlockNumberOrTag::Latest)
        .await?
        .ok_or_else(|| eyre::eyre!("latest block not found"))?;
    let ts = block.header.timestamp;
    Ok(ts)
}
