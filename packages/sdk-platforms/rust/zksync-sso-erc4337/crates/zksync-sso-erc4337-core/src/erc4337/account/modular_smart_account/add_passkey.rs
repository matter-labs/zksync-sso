use crate::erc4337::{
    account::{
        erc7579::{Execution, calls::encode_calls},
        modular_smart_account::{
            WebAuthnValidator,
            send::{SendParams, send_transaction},
        },
    },
    bundler::pimlico::client::BundlerClient,
    signer::Signer,
};
use alloy::{
    primitives::{Address, Bytes, U256},
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

pub async fn add_passkey<P: Provider + Send + Sync + Clone>(
    account_address: Address,
    passkey: PasskeyPayload,
    webauthn_validator: Address,
    entry_point_address: Address,
    provider: P,
    bundler_client: BundlerClient,
    signer: Signer,
) -> eyre::Result<()> {
    let call_data = add_passkey_call_data(passkey, webauthn_validator);

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

fn add_passkey_call_data(
    passkey: PasskeyPayload,
    webauthn_validator: Address,
) -> Bytes {
    let add_validation_key_call_data = add_validation_key_call_data(passkey);

    let call = {
        let target = webauthn_validator;
        let value = U256::from(0);
        let data = add_validation_key_call_data;
        Execution { target, value, data }
    };

    let calls = vec![call];
    encode_calls(calls).into()
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
                erc7579::{
                    add_module::add_module,
                    module_installed::is_module_installed,
                },
                modular_smart_account::{
                    deploy::{DeployAccountParams, EOASigners, deploy_account},
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
        })
        .await?;

        println!("Account deployed");

        let is_module_installed = is_module_installed(
            eoa_validator_address,
            address,
            provider.clone(),
        )
        .await?;
        eyre::ensure!(is_module_installed, "Module is not installed");
        fund_account_with_default_amount(address, provider.clone()).await?;

        let webauthn_module = contracts.webauthn_validator;
        {
            let signer = create_eoa_signer(
                signer_private_key.clone(),
                eoa_validator_address,
            )?;

            add_module(
                address,
                webauthn_module,
                entry_point_address,
                provider.clone(),
                bundler_client.clone(),
                signer,
            )
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

            add_passkey(
                address,
                passkey,
                webauthn_module,
                entry_point_address,
                provider,
                bundler_client,
                signer,
            )
            .await?;
        }

        println!("Passkey successfully added");

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
