mod tests {
    use crate::{
        erc4337::{
            account::{
                erc7579::module::{
                    Module,
                    add::{AddModuleParams, AddModulePayload, add_module},
                    installed::{IsModuleInstalledParams, is_module_installed},
                },
                modular_smart_account::{
                    deploy::{
                        DeployAccountParams, EOASigners,
                        user_op::{
                            DeployAccountWithUserOpParams,
                            deploy_account_with_user_op,
                        },
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
            signer::create_eoa_signer,
        },
        utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::primitives::{U256, Uint, address};
    use alloy_provider::ProviderBuilder;

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
}
