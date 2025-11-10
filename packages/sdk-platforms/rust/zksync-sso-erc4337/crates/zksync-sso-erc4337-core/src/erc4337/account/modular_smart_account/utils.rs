use alloy::{
    primitives::Address,
    providers::Provider,
    rpc::types::{BlockNumberOrTag, Filter, FilterSet, Log},
    sol_types::SolEvent,
};
use std::collections::HashSet;

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
    P: Provider + Send + Sync + Clone,
{
    // Increase time by the specified number of seconds
    let _: u64 =
        provider.raw_request("evm_increaseTime".into(), (seconds,)).await?;

    // Mine a block to apply the time change
    // evm_mine returns the new block number as a hex string
    let block_number: String =
        provider.raw_request("evm_mine".into(), ()).await?;

    Ok(block_number)
}
