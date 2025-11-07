use crate::{
    config::contracts::Contracts,
    erc4337::account::modular_smart_account::{
        signers::eoa::EOAKeyValidator,
        utils::{
            calculate_from_block, create_logs_filter_with_range,
            parse_add_remove_events,
        },
    },
};
use alloy::{
    primitives::Address, providers::Provider, rpc::types::Log,
    sol_types::SolEvent,
};
use std::collections::HashSet;

pub async fn get_active_owners<P>(
    account_address: Address,
    provider: P,
    contracts: Contracts,
) -> eyre::Result<Vec<Address>>
where
    P: Provider + Send + Sync + Clone,
{
    let eoa_validator_address = contracts.eoa_validator;

    let from_block = {
        let current_block = provider.get_block_number().await?;
        calculate_from_block(current_block)
    };

    let filter =
        create_logs_filter_with_range(eoa_validator_address, from_block);

    let all_logs = provider.get_logs(&filter).await?;

    let (added_signers, removed_signers) =
        parse_eoa_events(all_logs, account_address);

    let active_signers = filter_active_signers(added_signers, removed_signers);

    Ok(active_signers)
}

fn parse_eoa_events(
    logs: Vec<Log>,
    account_address: Address,
) -> (Vec<Address>, HashSet<Address>) {
    let owner_added_topic = EOAKeyValidator::OwnerAdded::SIGNATURE_HASH;
    let owner_removed_topic = EOAKeyValidator::OwnerRemoved::SIGNATURE_HASH;

    parse_add_remove_events(
        logs,
        account_address,
        owner_added_topic,
        owner_removed_topic,
        |event: EOAKeyValidator::OwnerAdded, account_address| {
            if event.smartAccount == account_address {
                Some(event.owner)
            } else {
                None
            }
        },
        |event: EOAKeyValidator::OwnerRemoved, account_address| {
            if event.smartAccount == account_address {
                Some(event.owner)
            } else {
                None
            }
        },
    )
}

fn filter_active_signers(
    added_signers: Vec<Address>,
    removed_signers: HashSet<Address>,
) -> Vec<Address> {
    added_signers
        .into_iter()
        .filter(|signer| !removed_signers.contains(signer))
        .collect()
}
