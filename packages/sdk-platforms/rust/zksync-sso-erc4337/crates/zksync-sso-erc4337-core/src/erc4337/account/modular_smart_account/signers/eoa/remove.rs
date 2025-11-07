use crate::erc4337::{
    account::{
        erc7579::{Execution, calls::encode_calls},
        modular_smart_account::{
            send::{SendParams, send_transaction},
            signers::eoa::EOAKeyValidator,
        },
    },
    bundler::pimlico::client::BundlerClient,
    signer::Signer,
};
use alloy::{
    primitives::{Address, Bytes, U256},
    providers::Provider,
    sol_types::SolCall,
};

pub async fn remove_owner<P>(
    account_address: Address,
    owner: Address,
    entry_point_address: Address,
    eoa_validator_address: Address,
    bundler_client: BundlerClient,
    provider: P,
    signer: Signer,
) -> eyre::Result<()>
where
    P: Provider + Send + Sync + Clone,
{
    let call_data = remove_owner_call_data(owner, eoa_validator_address);

    send_transaction(SendParams {
        account: account_address,
        entry_point: entry_point_address,
        factory_payload: None,
        call_data,
        nonce_key: None,
        paymaster: None,
        bundler_client,
        provider,
        signer,
    })
    .await?;

    Ok(())
}

fn remove_owner_call_data(
    owner: Address,
    eoa_validator_address: Address,
) -> Bytes {
    let remove_owner_calldata =
        EOAKeyValidator::removeOwnerCall { owner }.abi_encode().into();

    let call = {
        let target = eoa_validator_address;
        let value = U256::from(0);
        let data = remove_owner_calldata;
        Execution { target, value, data }
    };

    let calls = vec![call];
    encode_calls(calls).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        erc4337::{
            account::{
                erc7579::module_installed::is_module_installed,
                modular_smart_account::{
                    deploy::{DeployAccountParams, EOASigners, deploy_account},
                    signers::eoa::{
                        active::get_active_owners, add::add_owner,
                        eoa_signature, stub_signature_eoa,
                    },
                    test_utilities::fund_account_with_default_amount,
                },
            },
            signer::Signer,
        },
        utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::primitives::{FixedBytes, address};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_remove_owner() -> eyre::Result<()> {
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

        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let eoa_validator_address = contracts.eoa_validator;
        let factory_address = contracts.account_factory;

        let initial_signer =
            address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
        let owner_to_remove =
            address!("0xb0Ee7A142d267C1f36714E4a8F75612F20a79721");

        let signers = vec![initial_signer];

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

        eyre::ensure!(is_eoa_module_installed, "eoa_module is not installed");

        fund_account_with_default_amount(account_address, provider.clone())
            .await?;

        // Add an owner first
        let signer = {
            let stub_sig = stub_signature_eoa(eoa_validator_address)?;
            let signer_private_key = signer_private_key.clone();
            let signature_provider = Arc::new(move |hash: FixedBytes<32>| {
                eoa_signature(&signer_private_key, eoa_validator_address, hash)
            });
            Signer { provider: signature_provider, stub_signature: stub_sig }
        };

        add_owner(
            account_address,
            owner_to_remove,
            entry_point_address,
            eoa_validator_address,
            bundler_client.clone(),
            provider.clone(),
            signer.clone(),
        )
        .await?;

        println!("Owner added");

        // Verify owner is added
        let active_signers_before = get_active_owners(
            account_address,
            provider.clone(),
            contracts.clone(),
        )
        .await?;

        println!("Active signers before removal: {:?}", active_signers_before);
        eyre::ensure!(
            active_signers_before.contains(&initial_signer),
            "Initial signer should be active"
        );
        eyre::ensure!(
            active_signers_before.contains(&owner_to_remove),
            "Owner to remove should be in active signers"
        );
        eyre::ensure!(
            active_signers_before.len() == 2,
            format!(
                "Expected 2 active signers, got {}",
                active_signers_before.len()
            )
        );

        // Remove the owner
        remove_owner(
            account_address,
            owner_to_remove,
            entry_point_address,
            eoa_validator_address,
            bundler_client.clone(),
            provider.clone(),
            signer,
        )
        .await?;

        println!("Owner removed");

        // Verify owner is removed
        let active_signers_after =
            get_active_owners(account_address, provider.clone(), contracts)
                .await?;

        println!("Active signers after removal: {:?}", active_signers_after);

        eyre::ensure!(
            active_signers_after.contains(&initial_signer),
            "Initial signer should still be active"
        );
        eyre::ensure!(
            !active_signers_after.contains(&owner_to_remove),
            "Removed owner should not be in active signers"
        );
        eyre::ensure!(
            active_signers_after.len() == 1,
            format!(
                "Expected 1 active signer, got {}",
                active_signers_after.len()
            )
        );

        println!("All assertions passed!");

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
