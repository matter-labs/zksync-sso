use alloy::primitives::{Address, Uint};

pub fn keyed_nonce(session_signer_address: Address) -> Uint<192, 3> {
    let nonce_bytes = session_signer_address.as_slice();
    Uint::from_be_slice(nonce_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        erc4337::{
            account::{
                erc7579::{
                    calls::{Execution, encode_calls, encoded_call_data},
                    module::{
                        Module,
                        add::{AddModuleParams, AddModulePayload, add_module},
                        installed::{
                            IsModuleInstalledParams, is_module_installed,
                        },
                    },
                },
                modular_smart_account::{
                    deploy::{DeployAccountParams, EOASigners, deploy_account},
                    send::{SendUserOpParams, send_user_op},
                    session::{
                        create::{CreateSessionParams, create_session},
                        hash::hash_session,
                        session_lib::session_spec::{
                            SessionSpec, limit_type::LimitType,
                            transfer_spec::TransferSpec,
                            usage_limit::UsageLimit,
                        },
                        signature::session_signature,
                    },
                    test_utilities::fund_account_with_default_amount,
                },
            },
            signer::{Signer, create_eoa_signer},
        },
        utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::{
        primitives::{Bytes, FixedBytes, U256, Uint, address, keccak256},
        signers::{SignerSync, local::PrivateKeySigner},
        sol_types::SolValue,
    };
    use std::{future::Future, pin::Pin, str::FromStr, sync::Arc};

    #[tokio::test]
    async fn test_keyed_nonce() -> eyre::Result<()> {
        let expected_keyed_nonce = Uint::from_str_radix(
            "CEbb58e4082Af6FaC6Ea275740f10073d1610ad9",
            16,
        )?;

        let session_signer_address =
            address!("0xCEbb58e4082Af6FaC6Ea275740f10073d1610ad9");

        let keyed_nonce = keyed_nonce(session_signer_address);

        eyre::ensure!(
            expected_keyed_nonce == keyed_nonce,
            "Keyed nonce mismatch, expected: {}, got: {}",
            expected_keyed_nonce,
            keyed_nonce
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_send_transaction_session() -> eyre::Result<()> {
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

        let session_key_module = contracts.session_validator;
        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;
        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let session_key_hex = "0xb1da23908ba44fb1c6147ac1b32a1dbc6e7704ba94ec495e588d1e3cdc7ca6f9";
        println!("\n\n\nsession_key_hex: {}", session_key_hex);
        let session_signer_address =
            PrivateKeySigner::from_str(session_key_hex)?.address();
        println!("session_key address: {}", session_signer_address);

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

        let is_eoa_module_installed =
            is_module_installed(IsModuleInstalledParams {
                module: Module::eoa_validator(eoa_validator_address),
                account: address,
                provider: provider.clone(),
            })
            .await?;

        eyre::ensure!(
            is_eoa_module_installed,
            "is_eoa_module_installed is not installed"
        );

        fund_account_with_default_amount(address, provider.clone()).await?;

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

        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        let expires_at = Uint::from(2088558400u64);
        let target = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
        let session_spec = SessionSpec {
            signer: session_signer_address,
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

        let session_hash = hash_session(session_spec.clone());
        let hash_to_sign = keccak256((session_hash, address).abi_encode());
        let signer_key = PrivateKeySigner::from_str(session_key_hex).unwrap();
        let signature = signer_key.sign_hash_sync(&hash_to_sign).unwrap();
        let proof = signature.as_bytes();

        create_session(CreateSessionParams {
            account_address: address,
            spec: session_spec.clone(),
            entry_point_address,
            session_key_validator: session_key_module,
            paymaster: None,
            bundler_client: bundler_client.clone(),
            provider: provider.clone(),
            signer: signer.clone(),
            proof: proof.into(),
        })
        .await?;

        println!("Session successfully created");

        // Send transaction using session signer
        let calldata = encoded_call_data(target, None, Some(U256::from(1)));

        let session_key_hex_owned = session_key_hex.to_string();
        let session_spec_arc = Arc::new(session_spec.clone());

        let session_signer = {
            let stub_sig = session_signature(
                &session_key_hex_owned,
                session_key_module,
                &session_spec,
                Default::default(),
            )?;
            let session_key_hex_arc = Arc::new(session_key_hex_owned.clone());
            let session_key_module_clone = session_key_module;
            let session_spec_arc_inner = Arc::clone(&session_spec_arc);

            let signature_provider = Arc::new(
                move |hash: FixedBytes<32>| -> Pin<
                    Box<dyn Future<Output = eyre::Result<Bytes>> + Send>,
                > {
                    let session_key_hex = Arc::clone(&session_key_hex_arc);
                    let session_spec = Arc::clone(&session_spec_arc_inner);
                    Box::pin(async move {
                        session_signature(
                            session_key_hex.as_str(),
                            session_key_module_clone,
                            &session_spec,
                            hash,
                        )
                    })
                        as Pin<
                            Box<
                                dyn Future<Output = eyre::Result<Bytes>> + Send,
                            >,
                        >
                },
            );

            Signer { provider: signature_provider, stub_signature: stub_sig }
        };

        let keyed_nonce = keyed_nonce(session_signer_address);

        send_user_op(SendUserOpParams {
            account: address,
            entry_point: entry_point_address,
            factory_payload: None,
            call_data: calldata,
            nonce_key: Some(keyed_nonce),
            paymaster: None,
            bundler_client,
            provider,
            signer: session_signer,
        })
        .await?;

        println!("Session transaction successfully sent");

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }

    /// Regression-style test: intentionally omit the keyed nonce used for session
    /// transactions to mirror the current failing E2E browser flow (which does not
    /// yet supply the session-specific nonce key). We expect estimation or validation
    /// to revert (AA23) rather than succeeding, and we assert on the error surface.
    #[tokio::test]
    async fn test_send_transaction_session_missing_keyed_nonce()
    -> eyre::Result<()> {
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

        let session_key_module = contracts.session_validator;
        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;
        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        // Deterministic session key used purely for testing.
        let session_key_hex = "0xb1da23908ba44fb1c6147ac1b32a1dbc6e7704ba94ec495e588d1e3cdc7ca6f9";
        let session_signer_address =
            PrivateKeySigner::from_str(session_key_hex)?.address();

        let eoa_signers = EOASigners {
            addresses: vec![address!(
                "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720"
            )],
            validator_address: eoa_validator_address,
        };

        let account_address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: Some(eoa_signers),
            webauthn_signer: None,
            id: None,
            provider: provider.clone(),
            session_validator: None,
        })
        .await?;

        fund_account_with_default_amount(account_address, provider.clone())
            .await?;

        // Install session key module via EOA signer.
        let eoa_signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;
        add_module(AddModuleParams {
            account_address,
            module: AddModulePayload::session_key(session_key_module),
            entry_point_address,
            paymaster: None,
            provider: provider.clone(),
            bundler_client: bundler_client.clone(),
            signer: eoa_signer.clone(),
        })
        .await?;

        // Create session spec (matches working Rust spec; target == recipient to isolate nonce issue).
        let expires_at = Uint::from(2088558400u64);
        let target = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
        let session_spec = SessionSpec {
            signer: session_signer_address,
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

        let session_hash = hash_session(session_spec.clone());
        let hash_to_sign =
            keccak256((session_hash, account_address).abi_encode());
        let signer_key = PrivateKeySigner::from_str(session_key_hex).unwrap();
        let signature = signer_key.sign_hash_sync(&hash_to_sign).unwrap();
        let proof = signature.as_bytes();

        create_session(CreateSessionParams {
            account_address,
            spec: session_spec.clone(),
            entry_point_address,
            session_key_validator: session_key_module,
            paymaster: None,
            bundler_client: bundler_client.clone(),
            provider: provider.clone(),
            signer: eoa_signer.clone(),
            proof: proof.into(),
        })
        .await?;

        // Build session signer (stub signature + provider closure) identical to success case.
        let stub_sig = session_signature(
            session_key_hex,
            session_key_module,
            &session_spec,
            Default::default(),
        )?;
        use alloy::primitives::FixedBytes; // local import for closure
        use std::{future::Future, pin::Pin, sync::Arc};
        let session_key_hex_owned = session_key_hex.to_string();
        let session_spec_arc = std::sync::Arc::new(session_spec.clone());
        let signature_provider = Arc::new(
            move |hash: FixedBytes<32>| -> Pin<
                Box<dyn Future<Output = eyre::Result<Bytes>> + Send>,
            > {
                let session_key_hex = session_key_hex_owned.clone();
                let session_spec = session_spec_arc.clone();
                Box::pin(async move {
                    session_signature(
                        &session_key_hex,
                        session_key_module,
                        &session_spec,
                        hash,
                    )
                })
            },
        );
        let session_signer =
            Signer { provider: signature_provider, stub_signature: stub_sig };

        // Prepare a simple value transfer call.
        let call = Execution {
            target,
            value: U256::from(1),
            call_data: Bytes::default(),
        };
        let calldata = encode_calls(vec![call]).into();

        // Intentionally DO NOT provide nonce_key (simulating browser path). Expect failure.
        let result = send_user_op(SendUserOpParams {
            account: account_address,
            entry_point: entry_point_address,
            factory_payload: None,
            call_data: calldata,
            nonce_key: None, // critical difference
            paymaster: None,
            bundler_client: bundler_client.clone(),
            provider: provider.clone(),
            signer: session_signer,
        })
        .await;

        eyre::ensure!(
            result.is_err(),
            "Expected session send without keyed nonce to fail, but it succeeded"
        );

        let err = result.unwrap_err();
        let err_str = format!("{err}");
        // Surface signal: should mention AA23 (reverted during validation) or estimation failure.
        // AA23 is the expected error code when session validation fails.
        eyre::ensure!(
            err_str.contains("AA23")
                || err_str.contains("User operation")
                || err_str.contains("estimate"),
            "Unexpected error content: {err_str}"
        );

        drop(anvil_instance);
        drop(bundler);
        Ok(())
    }
}
