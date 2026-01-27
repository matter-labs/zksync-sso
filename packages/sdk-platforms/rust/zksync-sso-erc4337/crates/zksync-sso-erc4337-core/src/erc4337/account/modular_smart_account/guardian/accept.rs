use crate::erc4337::account::modular_smart_account::guardian::contract::GuardianExecutor;
use alloy::{
    primitives::Address, providers::Provider, rpc::types::TransactionReceipt,
};

#[derive(Clone)]
pub struct AcceptGuardianParams<P: Provider + Send + Sync + Clone> {
    pub guardian_executor: Address,
    pub account: Address,
    pub guardian_provider: P,
}

pub async fn accept_guardian<P>(
    params: AcceptGuardianParams<P>,
) -> eyre::Result<TransactionReceipt>
where
    P: Provider + Send + Sync + Clone,
{
    let AcceptGuardianParams { guardian_executor, account, guardian_provider } =
        params;

    let guardian_executor_instance =
        GuardianExecutor::new(guardian_executor, guardian_provider);

    let receipt = guardian_executor_instance
        .acceptGuardian(account)
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
                        list::get_guardians_list,
                        propose::{ProposeGuardianParams, propose_guardian},
                        status::get_guardian_status,
                    },
                    test_utilities::fund_account_with_default_amount,
                },
            },
            signer::create_eoa_signer,
        },
        utils::alloy_utilities::{
            ethereum_wallet_from_private_key,
            test_utilities::{
                config::TestInfraConfig,
                start_node_and_deploy_contracts_and_start_bundler_with_config,
            },
        },
    };
    use alloy::{primitives::address, providers::ProviderBuilder};

    #[tokio::test]
    async fn test_accept_guardian() -> eyre::Result<()> {
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

        println!("\n\n\n\n\n\n\nGuardian proposed\n\n\n\n\n\n");

        fund_account_with_default_amount(guardian_address, provider.clone())
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

        println!("\n\n\n\n\n\n\nGuardian accepted\n\n\n\n\n\n");

        // Verify guardian was accepted correctly
        {
            let guardians = get_guardians_list(
                account_address,
                guardian_module,
                provider.clone(),
            )
            .await?;
            eyre::ensure!(
                guardians.len() == 1,
                "Expected exactly one guardian in the list"
            );
            eyre::ensure!(
                guardians[0] == guardian_address,
                "Guardian address mismatch"
            );

            let status = get_guardian_status(
                account_address,
                guardian_address,
                guardian_module,
                provider.clone(),
            )
            .await?;
            eyre::ensure!(status.is_active(), "Guardian should be active");
        }

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
