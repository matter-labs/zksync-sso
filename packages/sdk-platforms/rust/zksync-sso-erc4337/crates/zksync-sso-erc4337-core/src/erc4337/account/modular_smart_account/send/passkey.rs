use crate::erc4337::{
    account::modular_smart_account::send::send_transaction as send_transaction_base,
    bundler::pimlico::client::BundlerClient,
    signer::{SignatureProvider, Signer},
};
use alloy::{
    primitives::{Address, Bytes, FixedBytes},
    providers::Provider,
};

pub async fn send_transaction<P: Provider + Send + Sync + Clone>(
    account: Address,
    _webauthn_validator: Address,
    entry_point: Address,
    call_data: Bytes,
    bundler_client: BundlerClient,
    provider: P,
    signature_provider: SignatureProvider,
) -> eyre::Result<()> {
    let stub_sig = signature_provider(FixedBytes::<32>::default())?;

    let signer =
        Signer { provider: signature_provider, stub_signature: stub_sig };

    send_transaction_base(
        account,
        entry_point,
        call_data,
        None,
        bundler_client,
        provider,
        signer,
    )
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
                deploy::{EOASigners, deploy_account},
                signature::{eoa_signature, stub_signature_eoa},
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

        let address = deploy_account(
            factory_address,
            Some(eoa_signers),
            None,
            provider.clone(),
        )
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

        send_transaction(
            address,
            webauthn_module,
            entry_point_address,
            calldata,
            bundler_client,
            provider.clone(),
            signature_provider,
        )
        .await?;

        println!("Passkey transaction successfully sent");

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
