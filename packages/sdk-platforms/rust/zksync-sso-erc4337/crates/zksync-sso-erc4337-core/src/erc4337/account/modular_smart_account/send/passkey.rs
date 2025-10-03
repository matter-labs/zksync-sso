use crate::erc4337::account::modular_smart_account::nonce::get_nonce;
use crate::erc4337::account::modular_smart_account::send::SignatureProvider;
use crate::erc4337::{
    account::{
        erc7579::{account::Execution, calls::encode_calls},
        modular_smart_account::send::send_transaction,
        modular_smart_account::signature::{eoa_signature, stub_signature_eoa},
    },
    bundler::models::estimate::Estimate,
    bundler::pimlico::client::BundlerClient,
    entry_point::EntryPoint,
    user_operation::hash::v08::get_user_operation_hash_entry_point,
};
use alloy::{
    primitives::{
        Address, Bytes, FixedBytes, U256, address, bytes, fixed_bytes,
    },
    providers::Provider,
    rpc::types::erc4337::{
        PackedUserOperation as AlloyPackedUserOperation, SendUserOperation,
    },
};
use alloy::{
    providers::ProviderBuilder, rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use alloy_provider::ext::Erc4337Api;
use std::str::FromStr;

pub async fn send_transaction_webauthn<P: Provider + Send + Sync + Clone>(
    account: Address,
    _webauthn_validator: Address,
    entry_point: Address,
    call_data: Bytes,
    bundler_client: BundlerClient,
    provider: P,
) -> eyre::Result<()> {
    let stub_sig =
        get_signature_from_js(FixedBytes::<32>::default().to_string())?;

    let signature_provider: SignatureProvider =
        Box::new(move |hash: FixedBytes<32>| {
            let result = get_signature_from_js(hash.to_string())?;
            Ok(result)
        });

    send_transaction(
        account,
        entry_point,
        call_data,
        bundler_client,
        provider,
        stub_sig,
        signature_provider,
    )
    .await
}

fn get_signature_from_js(hash: String) -> eyre::Result<Bytes> {
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
    let last_line =
        stdout.lines().filter(|line| !line.is_empty()).last().ok_or_else(
            || eyre::eyre!("No output from sign_hash_with_passkey command"),
        )?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::account::erc7579::add_module::add_module_call_data;
    use crate::erc4337::account::modular_smart_account::add_passkey::add_passkey_call_data;
    use crate::erc4337::account::modular_smart_account::deploy::{
        EOASigners, deploy_account_basic, is_module_installed,
    };
    use crate::erc4337::account::modular_smart_account::send::send_transaction_eoa;
    use alloy::{
        primitives::{Bytes, U256, address, bytes, fixed_bytes},
        providers::ProviderBuilder,
        rpc::types::TransactionRequest,
        signers::local::PrivateKeySigner,
    };
    use std::str::FromStr;

    #[tokio::test]
    async fn test_get_signature_from_js() -> eyre::Result<()> {
        // Test with a sample hash
        let test_hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();

        let signature = get_signature_from_js(test_hash)?;

        let expected_signature_hex = "0xf3f924c9badf6891d3676cfe9bf72e2c78527e1700000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000010031407dc6d0f38a21256dcaf23709f039d97816d1e76c6e5cf0343b7d84edc61977f7e162b597b925a912597ba3f809319c45b4e6092daf0c2f967198d7202fa200000000000000000000000000000000000000000000000000000000000001c00000000000000000000000000000000000000000000000000000000000000025a379a6f6eeafb9a55e378c118034e2751e682fab9f2d30ab13d2125586ce1947050000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000847b2274797065223a22776562617574686e2e676574222c226368616c6c656e6765223a22456a5257654a43727a6538534e465a346b4b764e37784930566e695171383376456a5257654a43727a6538222c226f726967696e223a2268747470733a2f2f6578616d706c652e636f6d222c2263726f73734f726967696e223a66616c73657d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000102868baa08431052f6c7541392a458f6400000000000000000000000000000000";
        let expected_signature = Bytes::from_str(expected_signature_hex)?;
        dbg!(expected_signature_hex.len()); // 1066
        dbg!(expected_signature.len());

        println!("len equal: {}", signature.len() == expected_signature.len());
        println!("expected len: {}", expected_signature.len());
        println!("actual len: {}", signature.len());
        eyre::ensure!(
            signature == expected_signature,
            "Signature mismatch, received: {signature:?}, expected: {expected_signature:?}"
        );

        // failures:

        // ---- erc4337::account::modular_smart_account::send::passkey::tests::test_get_signature_from_js stdout ----
        // Error: Failed to sign hash with passkey: Invalid hash

        Ok(())
    }

    #[tokio::test]
    async fn test_send_transaction_webauthn() -> eyre::Result<()> {
        let rpc_url = "http://localhost:8545".parse()?;

        let factory_address =
            address!("0x679FFF51F11C3f6CaC9F2243f9D14Cb1255F65A3");

        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let eoa_validator_address =
            address!("0x00427eDF0c3c3bd42188ab4C907759942Abebd93");

        let signer_private_key = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";

        let provider = {
            let signer = PrivateKeySigner::from_str(&signer_private_key)?;
            let alloy_signer = signer.clone();
            let ethereum_wallet =
                alloy::network::EthereumWallet::new(alloy_signer.clone());

            let provider = ProviderBuilder::new()
                .wallet(ethereum_wallet.clone())
                .connect_http(rpc_url);

            provider
        };

        let signers =
            vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let address = deploy_account_basic(
            factory_address,
            Some(eoa_signers),
            provider.clone(),
        )
        .await?;

        println!("Account deployed");
        // let address = address!("0x6bf1c0c174e11b933e7d8940afadf8bb7b8d421c");

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

        let webauthn_module =
            address!("0xF3F924c9bADF6891D3676cfe9bF72e2C78527E17");
        let module_type_id = 1;
        let init_data = Bytes::default();
        let call_data =
            add_module_call_data(webauthn_module, module_type_id, init_data);

        let bundler_client = {
            use crate::erc4337::bundler::config::BundlerConfig;
            let bundler_url = "http://localhost:4337".to_string();
            let config = BundlerConfig::new(bundler_url);
            BundlerClient::new(config)
        };

        let _ = send_transaction_eoa(
            address,
            eoa_validator_address,
            entry_point_address,
            call_data,
            bundler_client.clone(),
            provider.clone(),
            signer_private_key.to_string(),
        )
        .await?;

        let is_web_authn_module_installed =
            is_module_installed(webauthn_module, address, provider.clone())
                .await?;

        eyre::ensure!(
            is_web_authn_module_installed,
            "is_web_authn_module is not installed"
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
        let passkey_call_data = add_passkey_call_data(
            credential_id,
            passkey,
            origin_domain,
            webauthn_module,
        );

        // Send transaction to add passkey
        let _ = send_transaction_eoa(
            address,
            eoa_validator_address,
            entry_point_address,
            passkey_call_data,
            bundler_client.clone(),
            provider.clone(),
            signer_private_key.to_string(),
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

        let _ = send_transaction_webauthn(
            address,
            webauthn_module,
            entry_point_address,
            calldata,
            bundler_client,
            provider.clone(),
        )
        .await?;

        println!("Passkey transaction successfully sent");

        Ok(())
    }
}
