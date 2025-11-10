use crate::erc4337::{
    account::modular_smart_account::send::{SendParams, send_transaction},
    bundler::pimlico::client::BundlerClient,
    paymaster::params::PaymasterParams,
    signer::create_eoa_signer,
};
use alloy::{
    primitives::{Address, Bytes, Uint},
    providers::Provider,
};

#[derive(Clone)]
pub struct EOASendParams<P: Provider + Send + Sync + Clone> {
    pub account: Address,
    pub entry_point: Address,
    pub call_data: Bytes,
    pub nonce_key: Option<Uint<192, 3>>,
    pub paymaster: Option<PaymasterParams>,
    pub bundler_client: BundlerClient,
    pub provider: P,
    pub eoa_validator: Address,
    pub private_key_hex: String,
}

pub async fn eoa_send_transaction<P: Provider + Send + Sync + Clone>(
    params: EOASendParams<P>,
) -> eyre::Result<()> {
    let EOASendParams {
        account,
        entry_point,
        call_data,
        nonce_key,
        paymaster,
        bundler_client,
        provider,
        eoa_validator,
        private_key_hex,
    } = params;

    let signer = create_eoa_signer(private_key_hex, eoa_validator)?;

    _ = send_transaction(SendParams {
        account,
        entry_point,
        factory_payload: None,
        call_data,
        nonce_key,
        paymaster,
        bundler_client,
        provider,
        signer,
    })
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        erc4337::{
            account::{
                erc7579::{
                    Execution, calls::encode_calls,
                    module_installed::is_module_installed,
                },
                modular_smart_account::{
                    deploy::{
                        DeployAccountParams, EOASigners, deploy_account,
                        user_op::{
                            DeployAccountWithUserOpParams,
                            deploy_account_with_user_op,
                        },
                    },
                    test_utilities::fund_account_with_default_amount,
                },
            },
            paymaster::mock_paymaster::deploy_mock_paymaster_and_deposit_amount,
        },
        utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::{
        primitives::{Bytes, U256, address},
        providers::ProviderBuilder,
    };

    #[tokio::test]
    async fn test_deploy_and_send_transaction() -> eyre::Result<()> {
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
            start_anvil_and_deploy_contracts_and_start_bundler_with_config(
                &TestInfraConfig {
                    signer_private_key: signer_private_key.clone(),
                },
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
            session_signer: None,
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

        let call = {
            let target = address;
            let value = U256::from(1);
            let data = Bytes::default();
            Execution { target, value, data }
        };

        let calls = vec![call];

        fund_account_with_default_amount(address, provider.clone()).await?;

        let encoded_calls: Bytes = encode_calls(calls).into();

        eoa_send_transaction(EOASendParams {
            account: address,
            entry_point: entry_point_address,
            call_data: encoded_calls.clone(),
            nonce_key: None,
            paymaster: None,
            bundler_client: bundler_client.clone(),
            provider: provider.clone(),
            eoa_validator: eoa_validator_address,
            private_key_hex: signer_private_key.to_string(),
        })
        .await?;

        // Send transaction with paymaster
        let (_, paymaster_address) = deploy_mock_paymaster_and_deposit_amount(
            U256::from(1000000000000000000u64),
            provider.clone(),
        )
        .await?;

        let paymaster_params: Option<PaymasterParams> =
            Some(PaymasterParams::default_paymaster(paymaster_address));

        let unfunded_provider =
            ProviderBuilder::new().connect_http(node_url.clone());

        eoa_send_transaction(EOASendParams {
            account: address,
            entry_point: entry_point_address,
            call_data: encoded_calls,
            nonce_key: None,
            paymaster: paymaster_params,
            bundler_client,
            provider: unfunded_provider,
            eoa_validator: eoa_validator_address,
            private_key_hex: signer_private_key.to_string(),
        })
        .await?;

        drop(bundler);
        drop(anvil_instance);

        Ok(())
    }

    #[tokio::test]
    async fn test_deploy_and_send_transaction_with_paymaster()
    -> eyre::Result<()> {
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
            start_anvil_and_deploy_contracts_and_start_bundler_with_config(
                &TestInfraConfig {
                    signer_private_key: signer_private_key.clone(),
                },
            )
            .await?
        };

        let unfunded_provider =
            ProviderBuilder::new().connect_http(node_url.clone());

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

        let (mock_paymaster, paymaster_address) =
            deploy_mock_paymaster_and_deposit_amount(
                U256::from(1_000_000_000_000_000_000u64),
                provider.clone(),
            )
            .await?;
        let paymaster_params =
            PaymasterParams::default_paymaster(paymaster_address);

        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        let address =
            deploy_account_with_user_op(DeployAccountWithUserOpParams {
                deploy_params: DeployAccountParams {
                    factory_address,
                    eoa_signers: Some(eoa_signers),
                    webauthn_signer: None,
                    id: None,
                    provider: unfunded_provider.clone(),
                },
                entry_point_address,
                bundler_client: bundler_client.clone(),
                signer,
                paymaster: Some(paymaster_params.clone()),
                nonce_key: None,
            })
            .await?;

        let is_module_installed = is_module_installed(
            eoa_validator_address,
            address,
            provider.clone(),
        )
        .await?;

        eyre::ensure!(is_module_installed, "Module is not installed");

        let call = {
            let target = address;
            let value = U256::from(1);
            let data = Bytes::default();
            Execution { target, value, data }
        };

        let calls = vec![call];

        fund_account_with_default_amount(address, provider.clone()).await?;

        let encoded_calls: Bytes = encode_calls(calls).into();

        eoa_send_transaction(EOASendParams {
            account: address,
            entry_point: entry_point_address,
            call_data: encoded_calls.clone(),
            nonce_key: None,
            paymaster: None,
            bundler_client: bundler_client.clone(),
            provider: provider.clone(),
            eoa_validator: eoa_validator_address,
            private_key_hex: signer_private_key.to_string(),
        })
        .await?;

        eoa_send_transaction(EOASendParams {
            account: address,
            entry_point: entry_point_address,
            call_data: encoded_calls,
            nonce_key: None,
            paymaster: Some(paymaster_params.clone()),
            bundler_client,
            provider: unfunded_provider,
            eoa_validator: eoa_validator_address,
            private_key_hex: signer_private_key.to_string(),
        })
        .await?;

        drop(mock_paymaster);
        drop(bundler);
        drop(anvil_instance);

        Ok(())
    }
}
