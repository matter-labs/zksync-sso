use crate::erc4337::account::modular_smart_account::{
    guardian::contract::GuardianExecutor,
    utils::{calculate_from_block, create_logs_filter_with_range},
};
use alloy::{
    primitives::Address, providers::Provider, rpc::types::Log,
    sol_types::SolEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecoveryStatus {
    Initialized,
    Finalized,
}

impl RecoveryStatus {
    pub fn is_initialized(&self) -> bool {
        matches!(self, RecoveryStatus::Initialized)
    }

    pub fn is_finalized(&self) -> bool {
        matches!(self, RecoveryStatus::Finalized)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecoveryEventType {
    Initiated,
    Finished,
    Discarded,
}

pub async fn get_recovery_status<P>(
    account_address: Address,
    guardian_address: Address,
    guardian_executor_address: Address,
    provider: P,
) -> eyre::Result<Option<RecoveryStatus>>
where
    P: Provider + Send + Sync + Clone,
{
    let from_block = {
        let current_block = provider.get_block_number().await?;
        calculate_from_block(current_block)
    };

    let filter =
        create_logs_filter_with_range(guardian_executor_address, from_block);

    let all_logs = provider.get_logs(&filter).await?;

    let recovery_events =
        parse_recovery_events(all_logs, account_address, guardian_address)?;

    let status = determine_recovery_status(recovery_events);

    Ok(status)
}

fn parse_recovery_events(
    logs: Vec<Log>,
    account_address: Address,
    guardian_address: Address,
) -> eyre::Result<Vec<RecoveryEventType>> {
    let mut events = Vec::new();

    for log in logs {
        let Some(topic0) = log.inner.topics().first() else {
            continue;
        };

        match *topic0 {
            GuardianExecutor::RecoveryInitiated::SIGNATURE_HASH => {
                if let Ok(decoded) =
                    log.log_decode::<GuardianExecutor::RecoveryInitiated>()
                {
                    let event = decoded.inner.data;
                    if event.account == account_address
                        && event.guardian == guardian_address
                    {
                        events.push(RecoveryEventType::Initiated);
                    }
                }
            }
            GuardianExecutor::RecoveryFinished::SIGNATURE_HASH => {
                if let Ok(decoded) =
                    log.log_decode::<GuardianExecutor::RecoveryFinished>()
                {
                    let event = decoded.inner.data;
                    if event.account == account_address {
                        events.push(RecoveryEventType::Finished);
                    }
                }
            }
            GuardianExecutor::RecoveryDiscarded::SIGNATURE_HASH => {
                if let Ok(decoded) =
                    log.log_decode::<GuardianExecutor::RecoveryDiscarded>()
                {
                    let event = decoded.inner.data;
                    if event.account == account_address {
                        events.push(RecoveryEventType::Discarded);
                    }
                }
            }
            _ => continue,
        }
    }

    Ok(events)
}

fn determine_recovery_status(
    events: Vec<RecoveryEventType>,
) -> Option<RecoveryStatus> {
    // The last event in the list is the most recent (logs come in chronological order)
    // Process from the end to find the most recent relevant event
    // Since there can only be one pending recovery per account at a time,
    // the most recent event determines the current status

    // Iterate backwards to find the most recent event
    let event_type = events.iter().next_back()?;
    match *event_type {
        RecoveryEventType::Finished => {
            // Recovery was finalized - this is the final state
            Some(RecoveryStatus::Finalized)
        }
        RecoveryEventType::Discarded => {
            // Recovery was discarded - no recovery in progress
            None
        }
        RecoveryEventType::Initiated => {
            // Recovery was initiated - check if there's a Finished/Discarded after it
            // If we're here, we haven't seen a Finished/Discarded yet, so recovery is initialized
            Some(RecoveryStatus::Initialized)
        }
    }
}
