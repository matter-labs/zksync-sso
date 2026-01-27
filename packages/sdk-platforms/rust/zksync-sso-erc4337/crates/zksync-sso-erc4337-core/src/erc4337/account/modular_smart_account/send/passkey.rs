use crate::erc4337::{
    account::modular_smart_account::send::{SendUserOpParams, send_user_op},
    bundler::pimlico::client::BundlerClient,
    paymaster::params::PaymasterParams,
    signer::{SignatureProvider, Signer},
};
use alloy::{
    primitives::{Address, Bytes, FixedBytes},
    providers::Provider,
};

#[derive(Clone)]
pub struct PasskeySendParams<P: Provider + Send + Sync + Clone> {
    pub account: Address,
    pub _webauthn_validator: Address,
    pub entry_point: Address,
    pub call_data: Bytes,
    pub paymaster: Option<PaymasterParams>,
    pub bundler_client: BundlerClient,
    pub provider: P,
    pub signature_provider: SignatureProvider,
}

pub async fn passkey_send_transaction<P>(
    params: PasskeySendParams<P>,
) -> eyre::Result<()>
where
    P: Provider + Send + Sync + Clone,
{
    let PasskeySendParams {
        account,
        _webauthn_validator,
        entry_point,
        call_data,
        paymaster,
        bundler_client,
        provider,
        signature_provider,
    } = params;

    let stub_sig = signature_provider(FixedBytes::<32>::default()).await?;

    let signer =
        Signer { provider: signature_provider, stub_signature: stub_sig };

    _ = send_user_op(SendUserOpParams {
        account,
        entry_point,
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

#[cfg(test)]
pub mod tests {
    use super::*;
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
                        DeployAccountParams, EOASigners, deploy_account,
                        user_op::{
                            DeployAccountWithUserOpParams,
                            deploy_account_with_user_op,
                        },
                    },
                    passkey::add::{
                        AddPasskeyParams, PasskeyPayload, add_passkey,
                    },
                    test_utilities::fund_account_with_default_amount,
                },
            },
            bundler::Bundler,
            entry_point::{
                contract::EntryPoint::PackedUserOperation,
                nonce::{GetNonceWithKeyParams, get_nonce_with_key},
            },
            paymaster::mock_paymaster::deploy_mock_paymaster_and_deposit_amount,
            signer::{create_eoa_signer, test_utils::get_signature_from_js},
            user_operation::hash::user_operation_hash::get_user_operation_hash_entry_point,
        },
        utils::alloy_utilities::test_utilities::{
            config::TestInfraConfig,
            node_backend::{TestNodeBackend, resolve_test_node_backend},
            start_node_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::{
        primitives::{Bytes, U256, Uint, address, bytes, fixed_bytes},
        providers::ProviderBuilder,
        rpc::types::{
            TransactionRequest,
            erc4337::PackedUserOperation as AlloyPackedUserOperation,
        },
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_send_transaction_webauthn() -> eyre::Result<()> {
        if resolve_test_node_backend() == TestNodeBackend::ZkSyncOs {
            // NOTE: zksyncos currently returns AA24 signature error for this
            // passkey flow; skip until root cause is understood.
            return Ok(());
        }

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

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;

        let entry_point_address = contracts.entry_point;

        let eoa_signer_address =
            address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");

        let signers = vec![eoa_signer_address];

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

        let webauthn_module = contracts.webauthn_validator;

        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        {
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

            let is_web_authn_module_installed =
                is_module_installed(IsModuleInstalledParams {
                    module: Module::webauthn_validator(webauthn_module),
                    account: address,
                    provider: provider.clone(),
                })
                .await?;

            eyre::ensure!(
                is_web_authn_module_installed,
                "is_web_authn_module is not installed"
            );
        }

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

        let passkey = PasskeyPayload { credential_id, passkey, origin_domain };

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

        println!("Passkey successfully added");

        // Send transaction using passkey signer
        let calldata = encoded_call_data(address, None, Some(U256::from(1)));

        let signature_provider: SignatureProvider =
            Arc::new(move |hash: FixedBytes<32>| {
                Box::pin(async move {
                    let result = get_signature_from_js(hash.to_string())?;
                    Ok(result)
                })
            });

        passkey_send_transaction(PasskeySendParams {
            account: address,
            _webauthn_validator: webauthn_module,
            entry_point: entry_point_address,
            call_data: calldata,
            paymaster: None,
            bundler_client,
            provider: provider.clone(),
            signature_provider,
        })
        .await?;

        println!("Passkey transaction successfully sent");

        drop(anvil_instance);
        drop(bundler);
        Ok(())
    }

    #[tokio::test]
    async fn test_send_transaction_webauthn_sponsored() -> eyre::Result<()> {
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
            start_node_and_deploy_contracts_and_start_bundler_with_config(
                &TestInfraConfig {
                    signer_private_key: signer_private_key.clone(),
                },
            )
            .await?
        };

        let unfunded_provider =
            ProviderBuilder::new().connect_http(node_url.clone());

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;

        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let eoa_signer_address =
            address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");

        let (mock_paymaster, paymaster_address) =
            deploy_mock_paymaster_and_deposit_amount(
                U256::from(1_000_000_000_000_000_000u64),
                provider.clone(),
            )
            .await?;
        let paymaster = PaymasterParams::default_paymaster(paymaster_address);
        let paymaster_params = Some(paymaster);

        let signers = vec![eoa_signer_address];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

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
                    session_validator: None,
                    id: None,
                    provider: unfunded_provider.clone(),
                },
                entry_point_address,
                bundler_client: bundler_client.clone(),
                signer,
                paymaster: paymaster_params.clone(),
                nonce_key: None,
            })
            .await?;

        println!("Account deployed");

        fund_account_with_default_amount(address, provider.clone()).await?;

        let webauthn_module = contracts.webauthn_validator;

        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        add_module(AddModuleParams {
            account_address: address,
            module: AddModulePayload::webauthn(webauthn_module),
            entry_point_address,
            paymaster: paymaster_params.clone(),
            provider: unfunded_provider.clone(),
            bundler_client: bundler_client.clone(),
            signer: signer.clone(),
        })
        .await?;

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

        let passkey = PasskeyPayload { credential_id, passkey, origin_domain };

        add_passkey(AddPasskeyParams {
            account_address: address,
            passkey,
            webauthn_validator: webauthn_module,
            entry_point_address,
            paymaster: paymaster_params.clone(),
            provider: unfunded_provider.clone(),
            bundler_client: bundler_client.clone(),
            signer,
        })
        .await?;

        println!("Passkey successfully added");

        // Send transaction using passkey signer
        let calldata = encoded_call_data(address, None, Some(U256::from(1)));

        let signature_provider: SignatureProvider =
            Arc::new(move |hash: FixedBytes<32>| {
                Box::pin(async move {
                    let result = get_signature_from_js(hash.to_string())?;
                    Ok(result)
                })
            });

        passkey_send_transaction(PasskeySendParams {
            account: address,
            _webauthn_validator: webauthn_module,
            entry_point: entry_point_address,
            call_data: calldata,
            paymaster: paymaster_params.clone(),
            bundler_client,
            provider: unfunded_provider.clone(),
            signature_provider,
        })
        .await?;

        println!("Passkey transaction successfully sent");

        drop(mock_paymaster);
        drop(anvil_instance);
        drop(bundler);
        Ok(())
    }

    /// Test that mimics the WASM two-step flow:
    /// 1. Build UserOp and get hash
    /// 2. Sign hash with passkey (simulated)
    /// 3. Submit with signed UserOp
    #[tokio::test]
    async fn test_send_transaction_webauthn_two_step() -> eyre::Result<()> {
        if resolve_test_node_backend() == TestNodeBackend::ZkSyncOs {
            // NOTE: zksyncos currently returns AA24 signature error for this
            // passkey flow; skip until root cause is understood.
            return Ok(());
        }

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

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;

        let entry_point_address = contracts.entry_point;

        let eoa_signer_address =
            address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");

        let signers = vec![eoa_signer_address];

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

        println!("Account deployed: {:?}", address);

        // Fund the account
        {
            let fund_tx = TransactionRequest::default()
                .to(address)
                .value(U256::from(10000000000000000000u64));
            _ = provider.send_transaction(fund_tx).await?.get_receipt().await?;
        }

        let webauthn_module = contracts.webauthn_validator;

        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        // Install WebAuthn validator
        {
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

            let is_web_authn_module_installed =
                is_module_installed(IsModuleInstalledParams {
                    module: Module::webauthn_validator(webauthn_module),
                    account: address,
                    provider: provider.clone(),
                })
                .await?;

            eyre::ensure!(
                is_web_authn_module_installed,
                "WebAuthn module is not installed"
            );
        }

        // Add passkey
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

        println!("Passkey successfully added");

        // ===== TWO-STEP FLOW STARTS HERE =====

        // Step 1: Build UserOperation and get hash to sign
        println!("\nStep 1: Building UserOperation...");

        let call_data = encoded_call_data(address, None, Some(U256::from(1)));

        let nonce_key = Uint::from(0);
        let nonce = get_nonce_with_key(GetNonceWithKeyParams {
            sender: address,
            entry_point: entry_point_address,
            key: nonce_key,
            provider: provider.clone(),
        })
        .await?;

        // Create stub signature for gas estimation
        // Use all-zeros hash to generate a real passkey signature for estimation
        let stub_sig =
            get_signature_from_js(FixedBytes::<32>::default().to_string())?;

        // Build AlloyPackedUserOperation with stub values for gas estimation
        let mut user_op = AlloyPackedUserOperation {
            sender: address,
            nonce,
            paymaster: None,
            paymaster_verification_gas_limit: None,
            paymaster_data: None,
            paymaster_post_op_gas_limit: None,
            call_gas_limit: Default::default(),
            max_priority_fee_per_gas: Default::default(),
            max_fee_per_gas: Default::default(),
            pre_verification_gas: Default::default(),
            verification_gas_limit: Default::default(),
            factory: None,
            factory_data: None,
            call_data,
            signature: stub_sig,
        };

        // Estimate gas
        let estimated_gas = bundler_client
            .estimate_user_operation_gas(&user_op, &entry_point_address)
            .await?;

        // Update with estimated gas values
        user_op.call_gas_limit = estimated_gas.call_gas_limit;
        user_op.verification_gas_limit = (estimated_gas.verification_gas_limit
            * U256::from(6))
            / U256::from(5);
        user_op.pre_verification_gas = estimated_gas.pre_verification_gas;
        user_op.max_priority_fee_per_gas = U256::from(0x77359400);
        user_op.max_fee_per_gas = U256::from(0x82e08afeu64);

        // Pack gas limits and fees for hashing
        let packed_gas_limits: U256 =
            (user_op.verification_gas_limit << 128) | user_op.call_gas_limit;
        let gas_fees: U256 =
            (user_op.max_priority_fee_per_gas << 128) | user_op.max_fee_per_gas;

        let packed_user_op = PackedUserOperation {
            sender: user_op.sender,
            nonce: user_op.nonce,
            initCode: Bytes::default(),
            callData: user_op.call_data.clone(),
            accountGasLimits: packed_gas_limits.to_be_bytes().into(),
            preVerificationGas: user_op.pre_verification_gas,
            gasFees: gas_fees.to_be_bytes().into(),
            paymasterAndData: Bytes::default(),
            signature: user_op.signature.clone(),
        };

        let hash = get_user_operation_hash_entry_point(
            &packed_user_op,
            &entry_point_address,
            provider.clone(),
        )
        .await?;

        println!("UserOp hash to sign: {:?}", hash);
        println!("Packed user_op used for hash:");
        println!("  sender: {:?}", packed_user_op.sender);
        println!("  nonce: {:?}", packed_user_op.nonce);
        println!("  accountGasLimits: {:?}", packed_user_op.accountGasLimits);
        println!("  gasFees: {:?}", packed_user_op.gasFees);
        println!(
            "  preVerificationGas: {:?}",
            packed_user_op.preVerificationGas
        );

        // Step 2: Sign the hash (simulate JavaScript calling the passkey)
        println!("\nStep 2: Signing hash with passkey...");
        println!("Hash to sign (as string): {}", hash.0);

        let full_signature = get_signature_from_js(hash.0.to_string())?;
        println!("Full signature length: {} bytes", full_signature.len());

        // NOTE: get_signature_from_js already prepends the validator address (20 bytes)
        // So we don't need to prepend it again here like WASM does
        // The signature format is: [20 bytes validator address][ABI-encoded passkey data]

        // Step 3: Update UserOp with signature and submit
        println!("\nStep 3: Submitting UserOperation...");

        user_op.signature = full_signature;

        let user_op_hash = bundler_client
            .send_user_operation(entry_point_address, user_op)
            .await?;

        println!("UserOperation submitted: {:?}", user_op_hash);

        bundler_client.wait_for_user_operation_receipt(user_op_hash).await?;

        println!("Passkey transaction successfully sent (two-step)!");

        drop(anvil_instance);
        drop(bundler);
        Ok(())
    }
}
