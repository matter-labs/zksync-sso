use crate::erc4337::{
    account::modular_smart_account::send::{
        PaymasterParams, SendParams, send_transaction as send_transaction_base,
    },
    bundler::pimlico::client::BundlerClient,
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

pub async fn send_transaction<P>(
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

    let stub_sig = signature_provider(FixedBytes::<32>::default())?;

    let signer =
        Signer { provider: signature_provider, stub_signature: stub_sig };

    send_transaction_base(SendParams {
        account,
        entry_point,
        call_data,
        nonce_key: None,
        paymaster,
        bundler_client,
        provider,
        signer,
    })
    .await
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        erc4337::account::{
            erc7579::{
                Execution, add_module::add_module, calls::encode_calls,
                module_installed::is_module_installed,
            },
            modular_smart_account::{
                add_passkey::{PasskeyPayload, add_passkey},
                deploy::{DeployAccountParams, EOASigners, deploy_account},
                signature::{eoa_signature, stub_signature_eoa},
                test_utilities::fund_account_with_default_amount,
            },
        },
        utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::{
        primitives::{Bytes, U256, address, bytes, fixed_bytes},
        rpc::types::TransactionRequest,
    };
    use std::{str::FromStr, sync::Arc};

    pub fn get_signature_from_js(hash: String) -> eyre::Result<Bytes> {
        use std::process::Command;

        let working_dir = "../../../../../erc4337-contracts";

        let output = Command::new("pnpm")
            .arg("tsx")
            .arg("test/integration/utils.ts")
            .arg("--hash")
            .arg(&hash)
            .current_dir(working_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eyre::bail!("Failed to sign hash with passkey: {}", stderr);
        }

        let stdout = String::from_utf8(output.stdout)?;
        dbg!(&stdout);

        // Extract the last non-empty line which should be the hex signature
        let last_line = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .next_back()
            .ok_or_else(|| {
                eyre::eyre!("No output from sign_hash_with_passkey command")
            })?;
        dbg!(&last_line);
        dbg!(last_line.len());

        let hex_sig = last_line.trim();
        dbg!(&hex_sig);
        dbg!(hex_sig.len());

        let bytes = Bytes::from_str(hex_sig)?;
        dbg!(&bytes);
        dbg!(&bytes.len());

        Ok(bytes)
    }

    #[tokio::test]
    async fn test_send_transaction_webauthn() -> eyre::Result<()> {
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

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;

        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

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
            session_signer: None,
            id: None,
            provider: provider.clone(),
        })
        .await?;

        println!("Account deployed");

        let is_eoa_module_installed = is_module_installed(
            eoa_validator_address,
            address,
            provider.clone(),
        )
        .await?;

        eyre::ensure!(
            is_eoa_module_installed,
            "is_eoa_module_installed is not installed"
        );

        fund_account_with_default_amount(address, provider.clone()).await?;

        let webauthn_module = contracts.webauthn_validator;

        let signer = {
            let stub_sig = stub_signature_eoa(eoa_validator_address)?;
            let signer_private_key = signer_private_key.clone();
            let signature_provider = Arc::new(move |hash: FixedBytes<32>| {
                eoa_signature(&signer_private_key, eoa_validator_address, hash)
            });
            Signer { provider: signature_provider, stub_signature: stub_sig }
        };

        {
            add_module(
                address,
                webauthn_module,
                entry_point_address,
                provider.clone(),
                bundler_client.clone(),
                signer.clone(),
            )
            .await?;

            let is_web_authn_module_installed =
                is_module_installed(webauthn_module, address, provider.clone())
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

        add_passkey(
            address,
            passkey,
            webauthn_module,
            entry_point_address,
            provider.clone(),
            bundler_client.clone(),
            signer,
        )
        .await?;

        println!("Passkey successfully added");

        // Send transaction using passkey signer
        let call = {
            let target = address;
            let value = U256::from(1);
            let data = Bytes::default();
            Execution { target, value, data }
        };

        let calls = vec![call];
        let calldata = encode_calls(calls).into();

        let signature_provider: SignatureProvider =
            Arc::new(move |hash: FixedBytes<32>| {
                let result = get_signature_from_js(hash.to_string())?;
                Ok(result)
            });

        send_transaction(PasskeySendParams {
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

    /// Test that mimics the WASM two-step flow:
    /// 1. Build UserOp and get hash
    /// 2. Sign hash with passkey (simulated)
    /// 3. Submit with signed UserOp
    #[tokio::test]
    async fn test_send_transaction_webauthn_two_step() -> eyre::Result<()> {
        use crate::erc4337::{
            account::modular_smart_account::nonce::get_nonce, bundler::Bundler,
            entry_point::EntryPoint::PackedUserOperation,
            user_operation::hash::v08::get_user_operation_hash_entry_point,
        };
        use alloy::rpc::types::erc4337::PackedUserOperation as AlloyPackedUserOperation;
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

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;

        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

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
            session_signer: None,
            id: None,
            provider: provider.clone(),
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

        let signer = {
            let stub_sig = stub_signature_eoa(eoa_validator_address)?;
            let signer_private_key = signer_private_key.clone();
            let signature_provider = Arc::new(move |hash: FixedBytes<32>| {
                eoa_signature(&signer_private_key, eoa_validator_address, hash)
            });
            Signer { provider: signature_provider, stub_signature: stub_sig }
        };

        // Install WebAuthn validator
        {
            add_module(
                address,
                webauthn_module,
                entry_point_address,
                provider.clone(),
                bundler_client.clone(),
                signer.clone(),
            )
            .await?;

            let is_web_authn_module_installed =
                is_module_installed(webauthn_module, address, provider.clone())
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

        add_passkey(
            address,
            passkey_payload,
            webauthn_module,
            entry_point_address,
            provider.clone(),
            bundler_client.clone(),
            signer,
        )
        .await?;

        println!("Passkey successfully added");

        // ===== TWO-STEP FLOW STARTS HERE =====

        // Step 1: Build UserOperation and get hash to sign
        println!("\nStep 1: Building UserOperation...");

        let call = {
            let target = address;
            let value = U256::from(1);
            let data = Bytes::default();
            Execution { target, value, data }
        };

        let calls = vec![call];
        let call_data: Bytes = encode_calls(calls).into();

        let nonce_key = alloy::primitives::Uint::from(0);
        let nonce =
            get_nonce(entry_point_address, address, nonce_key, &provider)
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
