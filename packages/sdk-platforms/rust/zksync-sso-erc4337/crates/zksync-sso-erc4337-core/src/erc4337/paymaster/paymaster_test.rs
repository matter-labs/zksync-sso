#[cfg(test)]
mod tests {
    use crate::{
        erc4337::{
            account::{
                erc7579::{
                    calls::encoded_call_data,
                    module::{
                        Module,
                        add::{AddModuleParams, AddModulePayload, add_module},
                        installed::{
                            IsModuleInstalledParams, is_module_installed,
                        },
                    },
                },
                modular_smart_account::{
                    deploy::{
                        DeployAccountParams, EOASigners,
                        user_op::{
                            DeployAccountWithUserOpParams,
                            deploy_account_with_user_op,
                        },
                    },
                    passkey::add::{
                        AddPasskeyParams, PasskeyPayload, add_passkey,
                    },
                    send::passkey::{
                        PasskeySendParams, passkey_send_transaction,
                    },
                    session::{
                        create::{
                            CreateSessionParams, create_session,
                            tests::generate_session_proof,
                        },
                        session_lib::session_spec::{
                            SessionSpec, limit_type::LimitType,
                            transfer_spec::TransferSpec,
                            usage_limit::UsageLimit,
                        },
                    },
                    test_utilities::fund_account_with_default_amount,
                },
            },
            paymaster::{
                mock_paymaster::deploy_mock_paymaster_and_deposit_amount,
                params::PaymasterParams,
            },
            signer::{
                SignatureProvider, create_eoa_signer,
                test_utils::get_signature_from_js,
            },
        },
        utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::{
        primitives::{U256, Uint, address, bytes, fixed_bytes},
        providers::{Provider, ProviderBuilder},
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_deploy_account_create_session_with_paymaster()
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
        let session_key_module = contracts.session_validator;
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
        let paymaster =
            Some(PaymasterParams::default_paymaster(paymaster_address));

        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        // sponsored account deployment
        let address =
            deploy_account_with_user_op(DeployAccountWithUserOpParams {
                deploy_params: DeployAccountParams {
                    factory_address,
                    eoa_signers: Some(eoa_signers),
                    webauthn_signer: None,
                    session_validator: None,
                    id: None,
                    provider: unfunded_provider.clone(),
                },
                entry_point_address,
                bundler_client: bundler_client.clone(),
                signer,
                paymaster: paymaster.clone(),
                nonce_key: None,
            })
            .await?;

        fund_account_with_default_amount(address, provider.clone()).await?;

        // add session module
        {
            let signer = create_eoa_signer(
                signer_private_key.clone(),
                eoa_validator_address,
            )?;

            add_module(AddModuleParams {
                account_address: address,
                module: AddModulePayload::session_key(session_key_module),
                entry_point_address,
                paymaster: None,
                provider: provider.clone(),
                bundler_client: bundler_client.clone(),
                signer,
            })
            .await?;

            let is_session_key_module_installed =
                is_module_installed(IsModuleInstalledParams {
                    module: Module::session_key_validator(session_key_module),
                    account: address,
                    provider: provider.clone(),
                })
                .await?;

            eyre::ensure!(
                is_session_key_module_installed,
                "session_key_module is not installed"
            );

            println!("\n\n\nsession_key_module successfully installed\n\n\n")
        }

        // create session with rich wallet
        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        {
            let signer_address =
                address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
            let expires_at = Uint::from(2088558400u64);
            let target = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
            let session_spec = SessionSpec {
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
            };

            // Calculate proof
            let proof = generate_session_proof(
                &session_spec,
                address,
                &signer_private_key,
            )?;

            create_session(CreateSessionParams {
                account_address: address,
                spec: session_spec,
                entry_point_address,
                session_key_validator: session_key_module,
                paymaster,
                bundler_client,
                provider: unfunded_provider,
                signer,
                proof,
            })
            .await?;
        }

        println!("Session successfully created");

        drop(mock_paymaster);
        drop(bundler);
        drop(anvil_instance);

        Ok(())
    }

    #[tokio::test]
    async fn test_passkey_eth_send_with_paymaster_no_fees() -> eyre::Result<()>
    {
        let (
            _node_url,
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

        // Deploy a mock paymaster up-front so deployment and later txs are sponsored
        let (_mock_paymaster, paymaster_address) =
            deploy_mock_paymaster_and_deposit_amount(
                U256::from(1_000_000_000_000_000_000u64),
                provider.clone(),
            )
            .await?;
        let paymaster =
            Some(PaymasterParams::default_paymaster(paymaster_address));

        // Deploy smart account with EOA signer
        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;
        let eoa_signers = EOASigners {
            addresses: vec![address!(
                "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720"
            )],
            validator_address: eoa_validator_address,
        };

        let unfunded_provider = alloy_provider::ProviderBuilder::new()
            .connect_http(_node_url.clone());

        let address =
            deploy_account_with_user_op(DeployAccountWithUserOpParams {
                deploy_params: DeployAccountParams {
                    factory_address,
                    eoa_signers: Some(eoa_signers),
                    webauthn_signer: None,
                    session_validator: None,
                    id: None,
                    provider: unfunded_provider.clone(),
                },
                entry_point_address,
                bundler_client: bundler_client.clone(),
                signer: create_eoa_signer(
                    signer_private_key.clone(),
                    eoa_validator_address,
                )?,
                paymaster: paymaster.clone(),
                nonce_key: None,
            })
            .await?;

        // Fund the account from rich wallet
        fund_account_with_default_amount(address, provider.clone()).await?;

        // Install WebAuthn validator and add a passkey
        let webauthn_module = contracts.webauthn_validator;
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
            signer: signer.clone(),
        })
        .await?;

        let is_webauthn_installed =
            is_module_installed(IsModuleInstalledParams {
                module: Module::webauthn_validator(webauthn_module),
                account: address,
                provider: provider.clone(),
            })
            .await?;

        eyre::ensure!(
            is_webauthn_installed,
            "WebAuthn module is not installed"
        );

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
        let passkey_payload =
            PasskeyPayload { credential_id, passkey, origin_domain };

        add_passkey(AddPasskeyParams {
            account_address: address,
            passkey: passkey_payload,
            webauthn_validator: webauthn_module,
            entry_point_address,
            paymaster: None,
            provider: provider.clone(),
            bundler_client: bundler_client.clone(),
            signer,
        })
        .await?;

        // Prepare recipient and transfer amount (avoid bundler utility key)
        let recipient = address!("0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC");
        let amount = U256::from(1000u64); // send 1000 wei for deterministic assertion

        let sender_balance_before = provider.get_balance(address).await?;
        let recipient_balance_before = provider.get_balance(recipient).await?;

        let calldata = encoded_call_data(recipient, None, Some(amount));

        // Signature provider (simulated passkey signing)
        let signature_provider: SignatureProvider = Arc::new(move |hash| {
            Box::pin(async move { get_signature_from_js(hash.to_string()) })
        });

        // Send transaction using passkey signer with paymaster covering fees
        passkey_send_transaction(PasskeySendParams {
            account: address,
            _webauthn_validator: webauthn_module,
            entry_point: entry_point_address,
            call_data: calldata,
            paymaster,
            bundler_client,
            provider: provider.clone(),
            signature_provider,
        })
        .await?;

        let sender_balance_after = provider.get_balance(address).await?;

        // Log detailed balance deltas to make mismatches easier to diagnose
        let sender_delta =
            sender_balance_before.saturating_sub(sender_balance_after);
        println!(
            "Sender balance delta: {sender_delta} (before: {sender_balance_before}, after: {sender_balance_after})"
        );

        // Assert only the transfer amount changed, no extra fees from sender
        eyre::ensure!(
            sender_delta == amount,
            "Sender paid more than the transfer amount (fees leaked)"
        );

        println!("Paymaster covered fees; only value transferred");

        drop(bundler);
        drop(anvil_instance);
        Ok(())
    }
}
