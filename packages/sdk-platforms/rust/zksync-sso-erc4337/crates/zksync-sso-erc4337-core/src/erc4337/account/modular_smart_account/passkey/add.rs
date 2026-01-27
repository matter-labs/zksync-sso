use crate::erc4337::{
    account::{
        erc7579::calls::encoded_call_with_target_and_data,
        modular_smart_account::{
            passkey::contract::WebAuthnValidator,
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
    sol,
    sol_types::SolCall,
};

sol! {
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    struct PasskeyPayload {
        bytes credential_id;
        bytes32[2] passkey;
        string origin_domain;
    }
}

pub struct AddPasskeyParams<P: Provider + Send + Sync + Clone> {
    pub account_address: Address,
    pub passkey: PasskeyPayload,
    pub webauthn_validator: Address,
    pub entry_point_address: Address,
    pub paymaster: Option<PaymasterParams>,
    pub provider: P,
    pub bundler_client: BundlerClient,
    pub signer: Signer,
}

pub async fn add_passkey<P>(params: AddPasskeyParams<P>) -> eyre::Result<()>
where
    P: Provider + Send + Sync + Clone,
{
    let AddPasskeyParams {
        account_address,
        passkey,
        webauthn_validator,
        entry_point_address,
        paymaster,
        provider,
        bundler_client,
        signer,
    } = params;

    let call_data = add_passkey_call_data(passkey, webauthn_validator);

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

pub fn add_passkey_call_data(
    passkey: PasskeyPayload,
    webauthn_validator: Address,
) -> Bytes {
    let add_validation_key_call_data = add_validation_key_call_data(passkey);
    encoded_call_with_target_and_data(
        webauthn_validator,
        add_validation_key_call_data,
    )
}

fn add_validation_key_call_data(passkey: PasskeyPayload) -> Bytes {
    let credential_id = passkey.credential_id;
    let origin_domain = passkey.origin_domain;
    let new_key = passkey.passkey;

    WebAuthnValidator::addValidationKeyCall {
        credentialId: credential_id,
        newKey: new_key,
        domain: origin_domain,
    }
    .abi_encode()
    .into()
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
    async fn test_add_passkey() -> eyre::Result<()> {
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

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;

        let signers =
            vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: Some(eoa_signers),
            webauthn_signer: None,
            id: None,
            provider: provider.clone(),
            session_validator: None,
        })
        .await?;

        println!("Account deployed");

        let is_module_installed =
            is_module_installed(IsModuleInstalledParams {
                module: Module::eoa_validator(eoa_validator_address),
                account: address,
                provider: provider.clone(),
            })
            .await?;
        eyre::ensure!(is_module_installed, "Module is not installed");
        fund_account_with_default_amount(address, provider.clone()).await?;

        let webauthn_module = contracts.webauthn_validator;
        {
            let signer = create_eoa_signer(
                signer_private_key.clone(),
                eoa_validator_address,
            )?;

            add_module(AddModuleParams {
                account_address: address,
                module: AddModulePayload::webauthn(webauthn_module),
                entry_point_address,
                paymaster: None,
                provider: provider.clone(),
                bundler_client: bundler_client.clone(),
                signer,
            })
            .await?;
        };

        {
            let credential_id = bytes!("0x2868baa08431052f6c7541392a458f64");
            let passkey = [
                fixed_bytes!(
                    "0xe0a43b9c64a2357ea7f66a0551f57442fbd32031162d9be762800864168fae40"
                ),
                fixed_bytes!(
                    "0x450875e2c28222e81eb25ae58d095a3e7ca295faa3fc26fb0e558a0b571da501"
                ),
            ];
            let origin_domain = "https://example.com".to_string();
            let passkey =
                PasskeyPayload { credential_id, passkey, origin_domain };

            let signer = create_eoa_signer(
                signer_private_key.clone(),
                eoa_validator_address,
            )?;

            add_passkey(AddPasskeyParams {
                account_address: address,
                passkey,
                webauthn_validator: webauthn_module,
                entry_point_address,
                paymaster: None,
                provider: provider.clone(),
                bundler_client: bundler_client.clone(),
                signer,
            })
            .await?;
        }

        println!("Passkey successfully added");

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
