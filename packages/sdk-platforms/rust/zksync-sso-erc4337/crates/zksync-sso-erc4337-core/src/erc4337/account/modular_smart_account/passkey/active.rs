use crate::{
    config::contracts::Contracts,
    erc4337::account::modular_smart_account::{
        passkey::contract::WebAuthnValidator,
        utils::{
            calculate_from_block, create_logs_filter_with_range,
            parse_add_remove_events,
        },
    },
};
use alloy::{
    primitives::{Address, Bytes},
    providers::Provider,
    rpc::types::Log,
    sol_types::SolEvent,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeyDetails {
    pub credential_id: Bytes,
    pub domain: String,
}

pub async fn get_active_passkeys<P>(
    account_address: Address,
    provider: P,
    contracts: Contracts,
) -> eyre::Result<Vec<PasskeyDetails>>
where
    P: Provider + Send + Sync + Clone,
{
    let webauthn_validator_address = contracts.webauthn_validator;

    let from_block = {
        let current_block = provider.get_block_number().await?;
        calculate_from_block(current_block)
    };

    let filter =
        create_logs_filter_with_range(webauthn_validator_address, from_block);

    let all_logs = provider.get_logs(&filter).await?;

    let (created_passkeys, removed_passkeys) =
        parse_passkey_events(all_logs, account_address);

    let active_passkeys =
        filter_active_passkeys(created_passkeys, removed_passkeys);

    Ok(active_passkeys)
}

fn parse_passkey_events(
    logs: Vec<Log>,
    account_address: Address,
) -> (Vec<PasskeyDetails>, HashSet<(Bytes, String)>) {
    let passkey_created_topic =
        WebAuthnValidator::PasskeyCreated::SIGNATURE_HASH;
    let passkey_removed_topic =
        WebAuthnValidator::PasskeyRemoved::SIGNATURE_HASH;

    parse_add_remove_events(
        logs,
        account_address,
        passkey_created_topic,
        passkey_removed_topic,
        |event: WebAuthnValidator::PasskeyCreated, account_address| {
            if event.keyOwner == account_address {
                Some(PasskeyDetails {
                    credential_id: event.credentialId,
                    domain: event.domain,
                })
            } else {
                None
            }
        },
        |event: WebAuthnValidator::PasskeyRemoved, account_address| {
            if event.keyOwner == account_address {
                Some((event.credentialId, event.domain))
            } else {
                None
            }
        },
    )
}

fn filter_active_passkeys(
    created_passkeys: Vec<PasskeyDetails>,
    removed_passkeys: HashSet<(Bytes, String)>,
) -> Vec<PasskeyDetails> {
    created_passkeys
        .into_iter()
        .filter(|passkey| {
            !removed_passkeys.contains(&(
                passkey.credential_id.clone(),
                passkey.domain.clone(),
            ))
        })
        .collect()
}
