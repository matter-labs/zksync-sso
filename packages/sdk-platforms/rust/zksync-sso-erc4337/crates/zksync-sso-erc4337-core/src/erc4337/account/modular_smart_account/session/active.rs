use crate::{
    config::contracts::Contracts,
    erc4337::account::modular_smart_account::{
        session::{
            SessionKeyValidator, session_lib::session_spec::SessionSpec,
        },
        utils::{
            calculate_from_block, create_logs_filter_with_range,
            parse_add_remove_events,
        },
    },
};
use alloy::{
    primitives::{Address, FixedBytes},
    providers::Provider,
    rpc::types::{Filter, Log},
    sol_types::SolEvent,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveSession {
    pub session_hash: FixedBytes<32>,
    pub session_spec: SessionSpec,
}

pub async fn get_active_sessions<P>(
    account_address: Address,
    provider: P,
    contracts: Contracts,
) -> eyre::Result<Vec<ActiveSession>>
where
    P: Provider + Send + Sync + Clone,
{
    let session_key_validator_address = contracts.session_validator;

    let from_block = {
        let current_block = provider.get_block_number().await?;
        calculate_from_block(current_block)
    };

    let filter = create_session_logs_filter_with_range(
        session_key_validator_address,
        from_block,
    );

    let all_logs = provider.get_logs(&filter).await?;

    let (created_sessions, revoked_hashes) =
        parse_session_events(all_logs, account_address);

    let active_sessions =
        filter_active_sessions(created_sessions, revoked_hashes);

    Ok(active_sessions)
}

fn create_session_logs_filter_with_range(
    session_key_validator_address: Address,
    from_block: u64,
) -> Filter {
    create_logs_filter_with_range(session_key_validator_address, from_block)
}

fn parse_session_events(
    logs: Vec<Log>,
    account_address: Address,
) -> (Vec<(FixedBytes<32>, SessionSpec)>, HashSet<FixedBytes<32>>) {
    let session_created_topic =
        SessionKeyValidator::SessionCreated::SIGNATURE_HASH;
    let session_revoked_topic =
        SessionKeyValidator::SessionRevoked::SIGNATURE_HASH;

    parse_add_remove_events(
        logs,
        account_address,
        session_created_topic,
        session_revoked_topic,
        |event: SessionKeyValidator::SessionCreated, account_address| {
            if event.account == account_address {
                let session_spec: SessionSpec = event.sessionSpec.into();
                Some((event.sessionHash, session_spec))
            } else {
                None
            }
        },
        |event: SessionKeyValidator::SessionRevoked, account_address| {
            if event.account == account_address {
                Some(event.sessionHash)
            } else {
                None
            }
        },
    )
}

pub fn filter_active_sessions(
    created_sessions: Vec<(FixedBytes<32>, SessionSpec)>,
    revoked_hashes: HashSet<FixedBytes<32>>,
) -> Vec<ActiveSession> {
    created_sessions
        .into_iter()
        .filter(|(hash, _)| !revoked_hashes.contains(hash))
        .map(|(session_hash, session_spec)| ActiveSession {
            session_hash,
            session_spec,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::contracts::Contracts,
        erc4337::{
            account::{
                erc7579::{
                    add_module::add_module,
                    module_installed::is_module_installed,
                },
                modular_smart_account::{
                    deploy::{DeployAccountParams, EOASigners, deploy_account},
                    session::{
                        create::create_session,
                        hash::hash_session,
                        revoke::revoke_session,
                        session_lib::session_spec::{
                            limit_type::LimitType, transfer_spec::TransferSpec,
                            usage_limit::UsageLimit,
                        },
                    },
                    test_utilities::fund_account_with_default_amount,
                },
            },
            signer::create_eoa_signer,
        },
        utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::{
        primitives::{FixedBytes, U256, Uint, address},
        rpc::types::{BlockNumberOrTag, FilterBlockOption, FilterSet},
    };
    use std::collections::HashSet;

    fn create_session_logs_filter(
        session_key_validator_address: Address,
    ) -> Filter {
        Filter {
            address: session_key_validator_address.into(),
            topics: [
                FilterSet::default(),
                FilterSet::default(),
                FilterSet::default(),
                FilterSet::default(),
            ],
            block_option: FilterBlockOption::Range {
                from_block: Some(BlockNumberOrTag::Earliest),
                to_block: None,
            },
        }
    }

    #[tokio::test]
    async fn test_get_active_sessions() -> eyre::Result<()> {
        let (
            _,
            anvil_instance,
            provider,
            contracts,
            signer_private_key,
            bundler,
            bundler_client,
        ) = {
            let signer_private_key = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6".to_string();
            let config = TestInfraConfig {
                signer_private_key: signer_private_key.clone(),
            };
            start_anvil_and_deploy_contracts_and_start_bundler_with_config(
                &config,
            )
            .await?
        };

        let session_key_module = contracts.session_validator;
        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;
        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let signers =
            vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let account_address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: Some(eoa_signers),
            webauthn_signer: None,
            id: None,
            provider: provider.clone(),
        })
        .await?;

        println!("Account deployed");

        let is_eoa_module_installed = is_module_installed(
            eoa_validator_address,
            account_address,
            provider.clone(),
        )
        .await?;

        eyre::ensure!(
            is_eoa_module_installed,
            "is_eoa_module_installed is not installed"
        );

        fund_account_with_default_amount(account_address, provider.clone())
            .await?;

        {
            let signer = create_eoa_signer(
                signer_private_key.clone(),
                eoa_validator_address,
            )?;

            add_module(
                account_address,
                session_key_module,
                entry_point_address,
                provider.clone(),
                bundler_client.clone(),
                signer,
            )
            .await?;

            let is_session_key_module_installed = is_module_installed(
                session_key_module,
                account_address,
                provider.clone(),
            )
            .await?;

            eyre::ensure!(
                is_session_key_module_installed,
                "session_key_module is not installed"
            );

            println!("\n\n\nsession_key_module successfully installed\n\n\n")
        }

        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        // Create first session (will remain active)
        let session_spec_1 = {
            let signer_address =
                address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
            let expires_at = Uint::from(2088558400u64);
            let target = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
            SessionSpec {
                signer: signer_address,
                expires_at,
                call_policies: vec![],
                fee_limit: UsageLimit {
                    limit_type: LimitType::Lifetime,
                    limit: U256::from(1_000_000_000_000_000_000u64),
                    period: Uint::from(0),
                },
                transfer_policies: vec![TransferSpec {
                    max_value_per_use: U256::from(1),
                    target,
                    value_limit: UsageLimit {
                        limit_type: LimitType::Unlimited,
                        limit: U256::from(0),
                        period: Uint::from(0),
                    },
                }],
            }
        };

        create_session(
            account_address,
            session_spec_1.clone(),
            entry_point_address,
            session_key_module,
            bundler_client.clone(),
            provider.clone(),
            signer.clone(),
        )
        .await?;

        println!("Session 1 created");

        // Create second session (will be revoked)
        let session_spec_2 = {
            let signer_address =
                address!("0xb0Ee7A142d267C1f36714E4a8F75612F20a79721");
            let expires_at = Uint::from(2088558400u64);
            let target = address!("0xb0Ee7A142d267C1f36714E4a8F75612F20a79721");
            SessionSpec {
                signer: signer_address,
                expires_at,
                call_policies: vec![],
                fee_limit: UsageLimit {
                    limit_type: LimitType::Lifetime,
                    limit: U256::from(2_000_000_000_000_000_000u64),
                    period: Uint::from(0),
                },
                transfer_policies: vec![TransferSpec {
                    max_value_per_use: U256::from(2),
                    target,
                    value_limit: UsageLimit {
                        limit_type: LimitType::Unlimited,
                        limit: U256::from(0),
                        period: Uint::from(0),
                    },
                }],
            }
        };

        create_session(
            account_address,
            session_spec_2.clone(),
            entry_point_address,
            session_key_module,
            bundler_client.clone(),
            provider.clone(),
            signer.clone(),
        )
        .await?;

        println!("Session 2 created");

        // Create third session (will remain active)
        let session_spec_3 = {
            let signer_address =
                address!("0xc0Ee7A142d267C1f36714E4a8F75612F20a79722");
            let expires_at = Uint::from(2088558400u64);
            let target = address!("0xc0Ee7A142d267C1f36714E4a8F75612F20a79722");
            SessionSpec {
                signer: signer_address,
                expires_at,
                call_policies: vec![],
                fee_limit: UsageLimit {
                    limit_type: LimitType::Lifetime,
                    limit: U256::from(3_000_000_000_000_000_000u64),
                    period: Uint::from(0),
                },
                transfer_policies: vec![TransferSpec {
                    max_value_per_use: U256::from(3),
                    target,
                    value_limit: UsageLimit {
                        limit_type: LimitType::Unlimited,
                        limit: U256::from(0),
                        period: Uint::from(0),
                    },
                }],
            }
        };

        create_session(
            account_address,
            session_spec_3.clone(),
            entry_point_address,
            session_key_module,
            bundler_client.clone(),
            provider.clone(),
            signer.clone(),
        )
        .await?;

        println!("Session 3 created");

        // Revoke session 2
        let session_hash_2 = hash_session(session_spec_2.clone());
        revoke_session(
            account_address,
            session_hash_2,
            entry_point_address,
            session_key_module,
            bundler_client.clone(),
            provider.clone(),
            signer.clone(),
        )
        .await?;

        println!("Session 2 revoked");

        // Get active sessions
        let active_sessions = get_active_sessions(
            account_address,
            provider.clone(),
            Contracts { session_validator: session_key_module, ..contracts },
        )
        .await?;

        println!("Active sessions count: {}", active_sessions.len());

        // Verify we have 2 active sessions (session 1 and session 3)
        eyre::ensure!(
            active_sessions.len() == 2,
            format!(
                "Expected 2 active sessions, got {}",
                active_sessions.len()
            )
        );

        // Verify session 1 is active
        let session_hash_1 = hash_session(session_spec_1.clone());
        let session_1_found =
            active_sessions.iter().any(|s| s.session_hash == session_hash_1);
        eyre::ensure!(
            session_1_found,
            "Session 1 should be active but was not found"
        );

        // Verify session 2 is NOT active (was revoked)
        let session_2_found =
            active_sessions.iter().any(|s| s.session_hash == session_hash_2);
        eyre::ensure!(
            !session_2_found,
            "Session 2 should not be active (was revoked) but was found"
        );

        // Verify session 3 is active
        let session_hash_3 = hash_session(session_spec_3.clone());
        let session_3_found =
            active_sessions.iter().any(|s| s.session_hash == session_hash_3);
        eyre::ensure!(
            session_3_found,
            "Session 3 should be active but was not found"
        );

        println!("All assertions passed!");

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }

    #[test]
    fn test_create_session_logs_filter() {
        let address = address!("0x1234567890123456789012345678901234567890");
        let filter = create_session_logs_filter(address);
        // Verify filter is created correctly
        assert!(matches!(
            filter.block_option,
            alloy::rpc::types::FilterBlockOption::Range { .. }
        ));
    }

    #[test]
    fn test_filter_active_sessions() {
        let hash1 = FixedBytes::from([1u8; 32]);
        let hash2 = FixedBytes::from([2u8; 32]);
        let hash3 = FixedBytes::from([3u8; 32]);

        let session_spec = SessionSpec {
            signer: address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720"),
            expires_at: Uint::from(2088558400u64),
            call_policies: vec![],
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(1_000_000_000_000_000_000u64),
                period: Uint::from(0),
            },
            transfer_policies: vec![],
        };

        let created_sessions = vec![
            (hash1, session_spec.clone()),
            (hash2, session_spec.clone()),
            (hash3, session_spec),
        ];

        let mut revoked_hashes = HashSet::new();
        revoked_hashes.insert(hash2); // Revoke session 2

        let active_sessions =
            filter_active_sessions(created_sessions, revoked_hashes);

        // Should have 2 active sessions (hash1 and hash3)
        assert_eq!(active_sessions.len(), 2);
        assert!(active_sessions.iter().any(|s| s.session_hash == hash1));
        assert!(!active_sessions.iter().any(|s| s.session_hash == hash2));
        assert!(active_sessions.iter().any(|s| s.session_hash == hash3));
    }
}
