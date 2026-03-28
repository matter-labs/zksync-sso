use crate::erc4337::{
    account::{
        erc7579::calls::encoded_call_with_target_and_data,
        modular_smart_account::{
            passkey::{active::PasskeyDetails, contract::WebAuthnValidator},
            send::{SendUserOpParams, send_user_op},
        },
    },
    bundler::pimlico::client::BundlerClient,
    paymaster::params::PaymasterParams,
    signer::Signer,
};
use alloy::{
    primitives::{Address, Bytes},
    providers::Provider,
    sol_types::SolCall,
};

pub struct RemovePasskeyParams<P: Provider + Send + Sync + Clone> {
    pub account_address: Address,
    pub passkey: PasskeyDetails,
    pub entry_point_address: Address,
    pub webauthn_validator_address: Address,
    pub paymaster: Option<PaymasterParams>,
    pub bundler_client: BundlerClient,
    pub provider: P,
    pub signer: Signer,
}

pub async fn remove_passkey<P>(
    params: RemovePasskeyParams<P>,
) -> eyre::Result<()>
where
    P: Provider + Send + Sync + Clone,
{
    let RemovePasskeyParams {
        account_address,
        passkey,
        entry_point_address,
        webauthn_validator_address,
        paymaster,
        bundler_client,
        provider,
        signer,
    } = params;

    let call_data =
        remove_passkey_call_data(passkey, webauthn_validator_address);

    send_user_op(SendUserOpParams {
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

    Ok(())
}

pub fn remove_passkey_call_data(
    passkey: PasskeyDetails,
    webauthn_validator_address: Address,
) -> Bytes {
    let credential_id = passkey.credential_id;
    let domain = passkey.domain;
    let calldata = WebAuthnValidator::removeValidationKeyCall {
        credentialId: credential_id,
        domain,
    }
    .abi_encode()
    .into();

    encoded_call_with_target_and_data(webauthn_validator_address, calldata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        erc4337::{
            account::{
                erc7579::module::{
                    Module,
                    add::{AddModuleParams, AddModulePayload, add_module},
                    installed::{IsModuleInstalledParams, is_module_installed},
                },
                modular_smart_account::{
                    deploy::{DeployAccountParams, EOASigners, deploy_account},
                    passkey::{
                        active::{PasskeyDetails, get_active_passkeys},
                        add::{AddPasskeyParams, PasskeyPayload, add_passkey},
                    },
                    test_utilities::fund_account_with_default_amount,
                },
            },
            signer::create_eoa_signer,
        },
        utils::alloy_utilities::test_utilities::{
            config::TestInfraConfig,
            start_node_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::primitives::{address, bytes, fixed_bytes};

    #[tokio::test]
    async fn test_remove_passkey() -> eyre::Result<()> {
        let (
            _,
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

        let eoa_validator_address = contracts.eoa_validator;
        let webauthn_validator_address = contracts.webauthn_validator;
        let factory_address = contracts.account_factory;

        let initial_signer =
            address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");

        let signers = vec![initial_signer];

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

        let is_eoa_module_installed =
            is_module_installed(IsModuleInstalledParams {
                module: Module::eoa_validator(eoa_validator_address),
                account: account_address,
                provider: provider.clone(),
            })
            .await?;

        eyre::ensure!(is_eoa_module_installed, "eoa_module is not installed");

        fund_account_with_default_amount(account_address, provider.clone())
            .await?;

        // Install WebAuthn module
        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        add_module(AddModuleParams {
            account_address,
            module: AddModulePayload::webauthn(webauthn_validator_address),
            entry_point_address,
            paymaster: None,
            provider: provider.clone(),
            bundler_client: bundler_client.clone(),
            signer: signer.clone(),
        })
        .await?;

        let is_webauthn_module_installed =
            is_module_installed(IsModuleInstalledParams {
                module: Module::webauthn_validator(webauthn_validator_address),
                account: account_address,
                provider: provider.clone(),
            })
            .await?;

        eyre::ensure!(
            is_webauthn_module_installed,
            "webauthn_module is not installed"
        );

        // Add a passkey first
        let credential_id_1 = bytes!("0x2868baa08431052f6c7541392a458f64");
        let passkey_1 = [
            fixed_bytes!(
                "0xe0a43b9c64a2357ea7f66a0551f57442fbd32031162d9be762800864168fae40"
            ),
            fixed_bytes!(
                "0x450875e2c28222e81eb25ae58d095a3e7ca295faa3fc26fb0e558a0b571da501"
            ),
        ];
        let domain_1 = "https://example.com".to_string();

        let passkey_payload_1 = PasskeyPayload {
            credential_id: credential_id_1.clone(),
            passkey: passkey_1,
            origin_domain: domain_1.clone(),
        };

        add_passkey(AddPasskeyParams {
            account_address,
            passkey: passkey_payload_1,
            entry_point_address,
            webauthn_validator: webauthn_validator_address,
            paymaster: None,
            bundler_client: bundler_client.clone(),
            provider: provider.clone(),
            signer: signer.clone(),
        })
        .await?;

        println!("Passkey 1 added");

        // Add a second passkey to verify we can have multiple
        let credential_id_2 = bytes!("0x3868baa08431052f6c7541392a458f65");
        let passkey_2 = [
            fixed_bytes!(
                "0xf0a43b9c64a2357ea7f66a0551f57442fbd32031162d9be762800864168fae41"
            ),
            fixed_bytes!(
                "0x550875e2c28222e81eb25ae58d095a3e7ca295faa3fc26fb0e558a0b571da502"
            ),
        ];
        let domain_2 = "https://example2.com".to_string();

        let passkey_payload_2 = PasskeyPayload {
            credential_id: credential_id_2.clone(),
            passkey: passkey_2,
            origin_domain: domain_2.clone(),
        };

        add_passkey(AddPasskeyParams {
            account_address,
            passkey: passkey_payload_2,
            entry_point_address,
            webauthn_validator: webauthn_validator_address,
            paymaster: None,
            bundler_client: bundler_client.clone(),
            provider: provider.clone(),
            signer: signer.clone(),
        })
        .await?;

        println!("Passkey 2 added");

        // Verify both passkeys are active
        let active_passkeys_before = get_active_passkeys(
            account_address,
            provider.clone(),
            contracts.clone(),
        )
        .await?;

        println!(
            "Active passkeys before removal: {:?}",
            active_passkeys_before
        );
        eyre::ensure!(
            active_passkeys_before.len() == 2,
            format!(
                "Expected 2 active passkeys, got {}",
                active_passkeys_before.len()
            )
        );

        let passkey_1_found = active_passkeys_before.iter().any(|p| {
            p.credential_id == credential_id_1 && p.domain == domain_1
        });
        eyre::ensure!(
            passkey_1_found,
            "Passkey 1 should be in active passkeys"
        );

        let passkey_2_found = active_passkeys_before.iter().any(|p| {
            p.credential_id == credential_id_2 && p.domain == domain_2
        });
        eyre::ensure!(
            passkey_2_found,
            "Passkey 2 should be in active passkeys"
        );

        // Remove passkey 1
        let passkey_to_remove = PasskeyDetails {
            credential_id: credential_id_1.clone(),
            domain: domain_1.clone(),
        };

        remove_passkey(RemovePasskeyParams {
            account_address,
            passkey: passkey_to_remove,
            entry_point_address,
            webauthn_validator_address,
            paymaster: None,
            bundler_client: bundler_client.clone(),
            provider: provider.clone(),
            signer,
        })
        .await?;

        println!("Passkey 1 removed");

        // Verify passkey 1 is removed but passkey 2 is still active
        let active_passkeys_after =
            get_active_passkeys(account_address, provider.clone(), contracts)
                .await?;

        println!("Active passkeys after removal: {:?}", active_passkeys_after);

        let passkey_1_still_active = active_passkeys_after.iter().any(|p| {
            p.credential_id == credential_id_1 && p.domain == domain_1
        });
        eyre::ensure!(
            !passkey_1_still_active,
            "Removed passkey 1 should not be in active passkeys"
        );

        let passkey_2_still_active = active_passkeys_after.iter().any(|p| {
            p.credential_id == credential_id_2 && p.domain == domain_2
        });
        eyre::ensure!(
            passkey_2_still_active,
            "Passkey 2 should still be active"
        );

        eyre::ensure!(
            active_passkeys_after.len() == 1,
            format!(
                "Expected 1 active passkey, got {}",
                active_passkeys_after.len()
            )
        );

        println!("All assertions passed!");

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
