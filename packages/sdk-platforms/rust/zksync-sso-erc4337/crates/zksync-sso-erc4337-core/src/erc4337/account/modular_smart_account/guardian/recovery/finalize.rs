use crate::erc4337::account::modular_smart_account::guardian::contract::GuardianExecutor;
use alloy::{
    primitives::{Address, Bytes},
    providers::Provider,
    rpc::types::TransactionReceipt,
};

#[derive(Clone)]
pub struct FinalizeRecoveryParams<P: Provider + Send + Sync + Clone> {
    pub guardian_executor: Address,
    pub account: Address,
    pub data: Bytes,
    pub guardian_provider: P,
}

pub async fn finalize_recovery<P>(
    params: FinalizeRecoveryParams<P>,
) -> eyre::Result<TransactionReceipt>
where
    P: Provider + Send + Sync + Clone,
{
    let FinalizeRecoveryParams {
        guardian_executor,
        account,
        data,
        guardian_provider,
    } = params;

    let guardian_executor_instance =
        GuardianExecutor::new(guardian_executor, guardian_provider);

    let receipt = guardian_executor_instance
        .finalizeRecovery(account, data)
        .send()
        .await?
        .get_receipt()
        .await?;

    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        erc4337::{
            account::{
                erc7579::module::add::{
                    AddModuleParams, AddModulePayload, add_module,
                },
                modular_smart_account::{
                    deploy::{DeployAccountParams, EOASigners, deploy_account},
                    guardian::{
                        accept::{AcceptGuardianParams, accept_guardian},
                        propose::{ProposeGuardianParams, propose_guardian},
                        recovery::{
                            RecoveryType,
                            initialize::{
                                InitializeRecoveryParams, initialize_recovery,
                            },
                            status::get_recovery_status,
                        },
                    },
                    signers::eoa::active::get_active_owners,
                    test_utilities::fund_account_with_default_amount,
                    utils::{advance_time, latest_block_timestamp},
                },
            },
            signer::create_eoa_signer,
        },
        utils::alloy_utilities::{
            ethereum_wallet_from_private_key,
            test_utilities::{
                config::TestInfraConfig,
                node_backend::{TestNodeBackend, resolve_test_node_backend},
                start_node_and_deploy_contracts_and_start_bundler_with_config,
            },
        },
    };
    use alloy::{
        primitives::address, providers::ProviderBuilder, sol_types::SolValue,
    };

    #[tokio::test]
    async fn test_finalize_recovery() -> eyre::Result<()> {
        if resolve_test_node_backend() == TestNodeBackend::ZkSyncOs {
            return Ok(());
        }

        let (
            node_url,
            anvil_instance,
            provider,
            contracts,
            signer_private_key,
            bundler,
            bundler_client,
        ) = {
            let config = TestInfraConfig::rich_wallet_9();
            start_node_and_deploy_contracts_and_start_bundler_with_config(
                &config,
            )
            .await?
        };

        let entry_point_address = contracts.entry_point;

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;

        let signer_address =
            address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
        let signers = vec![signer_address];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let account_address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: Some(eoa_signers),
            webauthn_signer: None,
            session_validator: None,
            id: None,
            provider: provider.clone(),
        })
        .await?;

        println!("Account deployed");

        fund_account_with_default_amount(account_address, provider.clone())
            .await?;

        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        // Install WebAuthn module
        {
            let webauthn_module = contracts.webauthn_validator;
            add_module(AddModuleParams {
                account_address,
                module: AddModulePayload::webauthn(webauthn_module),
                entry_point_address,
                paymaster: None,
                provider: provider.clone(),
                bundler_client: bundler_client.clone(),
                signer: signer.clone(),
            })
            .await?;
        }

        let guardian_module = contracts.guardian_executor;

        // Install Guardian module
        add_module(AddModuleParams {
            account_address,
            module: AddModulePayload::guardian(guardian_module),
            entry_point_address,
            paymaster: None,
            provider: provider.clone(),
            bundler_client: bundler_client.clone(),
            signer: signer.clone(),
        })
        .await?;

        // Anvil Rich Wallet #8 as guardian
        let guardian_address =
            address!("0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f");
        let guardian_private_key = "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97".to_string();
        let guardian_wallet =
            ethereum_wallet_from_private_key(&guardian_private_key)?;
        let guardian_provider = ProviderBuilder::new()
            .wallet(guardian_wallet)
            .connect_http(node_url.clone());

        // Propose guardian
        propose_guardian(ProposeGuardianParams {
            guardian_executor: guardian_module,
            new_guardian: guardian_address,
            account: account_address,
            entry_point: entry_point_address,
            paymaster: None,
            provider: provider.clone(),
            bundler_client: bundler_client.clone(),
            signer: signer.clone(),
        })
        .await?;

        fund_account_with_default_amount(guardian_address, provider.clone())
            .await?;

        // Accept guardian
        accept_guardian(AcceptGuardianParams {
            guardian_executor: guardian_module,
            account: account_address,
            guardian_provider: guardian_provider.clone(),
        })
        .await?;

        // New owner: Alloy Rich Wallet #7
        let new_owner_address =
            address!("0x14dC79964da2C08b23698B3D3cc7Ca32193d9955");

        // Initialize recovery directly
        let initialize_recovery_data: Bytes =
            new_owner_address.abi_encode().into();
        {
            initialize_recovery(InitializeRecoveryParams {
                guardian_executor: guardian_module,
                account: account_address,
                recovery_type: RecoveryType::EOA,
                data: initialize_recovery_data.clone(),
                guardian_provider: guardian_provider.clone(),
            })
            .await?;
        }

        println!("\n\n\n\n\n\n\n\nRecovery initialized\n\n\n\n\n\n\n\n");

        // Advance time beyond the required delay for this recovery request.
        {
            let guardian_executor_instance =
                GuardianExecutor::new(guardian_module, provider.clone());
            let recovery = guardian_executor_instance
                .pendingRecovery(account_address)
                .call()
                .await?;
            let delay =
                guardian_executor_instance.REQUEST_DELAY_TIME().call().await?;
            let delay: u64 = delay.try_into().unwrap_or(u64::MAX);
            let recovery_ts: u64 =
                recovery.timestamp.try_into().unwrap_or(u64::MAX);
            let target_ts = recovery_ts.saturating_add(delay).saturating_add(1);
            let current_ts = latest_block_timestamp(&provider).await?;
            let delta = target_ts.saturating_sub(current_ts);
            if delta > 0 {
                advance_time(&provider, delta).await?;
            }
        }

        // Finalize recovery
        finalize_recovery(FinalizeRecoveryParams {
            guardian_executor: guardian_module,
            account: account_address,
            data: initialize_recovery_data,
            guardian_provider: guardian_provider.clone(),
        })
        .await?;

        // Verify recovery was finalized correctly
        {
            let recovery_status = get_recovery_status(
                account_address,
                guardian_address,
                guardian_module,
                provider.clone(),
            )
            .await?
            .ok_or(eyre::eyre!("Recovery status not found"))?;

            eyre::ensure!(
                recovery_status.is_finalized(),
                "Recovery should be finalized"
            );

            // Verify new owner was added
            let active_owners = get_active_owners(
                account_address,
                provider.clone(),
                contracts.clone(),
            )
            .await?;

            eyre::ensure!(
                active_owners.contains(&new_owner_address),
                "New owner should be added to the account"
            );
        }

        println!("\n\n\n\n\n\n\n\nRecovery finalized\n\n\n\n\n\n\n\n");

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
