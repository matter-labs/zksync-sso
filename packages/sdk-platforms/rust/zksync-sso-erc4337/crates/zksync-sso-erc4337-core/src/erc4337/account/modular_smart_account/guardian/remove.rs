use crate::erc4337::{
    account::{
        erc7579::calls::encoded_call_with_target_and_data,
        modular_smart_account::{
            guardian::contract::GuardianExecutor,
            send::{SendUserOpParams, send_user_op},
        },
    },
    bundler::{
        models::receipt::UserOperationReceipt, pimlico::client::BundlerClient,
    },
    paymaster::params::PaymasterParams,
    signer::Signer,
};
use alloy::{
    primitives::{Address, Bytes},
    providers::Provider,
    sol_types::SolCall,
};

#[derive(Clone)]
pub struct RemoveGuardianParams<P: Provider + Send + Sync + Clone> {
    pub guardian_executor: Address,
    pub guardian_to_remove: Address,
    pub account_address: Address,
    pub entry_point_address: Address,
    pub paymaster: Option<PaymasterParams>,
    pub provider: P,
    pub bundler_client: BundlerClient,
    pub signer: Signer,
}

pub async fn remove_guardian<P>(
    params: RemoveGuardianParams<P>,
) -> eyre::Result<UserOperationReceipt>
where
    P: Provider + Send + Sync + Clone,
{
    let RemoveGuardianParams {
        guardian_executor,
        guardian_to_remove,
        account_address,
        entry_point_address,
        paymaster,
        provider,
        bundler_client,
        signer,
    } = params;

    let call_data =
        remove_guardian_call_data(guardian_to_remove, guardian_executor);

    let receipt = send_user_op(SendUserOpParams {
        account: account_address,
        entry_point: entry_point_address,
        factory_payload: None,
        call_data,
        nonce_key: None,
        paymaster,
        bundler_client,
        provider,
        signer,
    })
    .await?;

    Ok(receipt)
}

pub fn remove_guardian_call_data(
    guardian_to_remove: Address,
    guardian_executor: Address,
) -> Bytes {
    let propose_guardian_calldata = GuardianExecutor::removeGuardianCall {
        guardianToRemove: guardian_to_remove,
    }
    .abi_encode()
    .into();

    encoded_call_with_target_and_data(
        guardian_executor,
        propose_guardian_calldata,
    )
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
                        list::get_guardians_list,
                        propose::{ProposeGuardianParams, propose_guardian},
                    },
                    test_utilities::fund_account_with_default_amount,
                },
            },
            signer::create_eoa_signer,
        },
        utils::alloy_utilities::{
            ethereum_wallet_from_private_key,
            test_utilities::{
                TestInfraConfig,
                start_anvil_and_deploy_contracts_and_start_bundler_with_config,
            },
        },
    };
    use alloy::{primitives::address, providers::ProviderBuilder};

    #[tokio::test]
    async fn test_remove_guardian() -> eyre::Result<()> {
        let (
            node_url,
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

        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

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

        // Accept guardian
        {
            let guardian_wallet =
                ethereum_wallet_from_private_key(&guardian_private_key)?;
            let guardian_provider = ProviderBuilder::new()
                .wallet(guardian_wallet)
                .connect_http(node_url);

            accept_guardian(AcceptGuardianParams {
                guardian_executor: guardian_module,
                account: account_address,
                guardian_provider,
            })
            .await?;
        }

        // Remove guardian
        remove_guardian(RemoveGuardianParams {
            guardian_executor: guardian_module,
            guardian_to_remove: guardian_address,
            account_address,
            entry_point_address,
            paymaster: None,
            provider: provider.clone(),
            bundler_client: bundler_client.clone(),
            signer: signer.clone(),
        })
        .await?;

        // Verify guardian was removed correctly
        {
            let guardians = get_guardians_list(
                account_address,
                guardian_module,
                provider.clone(),
            )
            .await?;
            eyre::ensure!(
                guardians.is_empty(),
                "Expected no guardians in the list, got {:?}",
                guardians
            );
        }

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
