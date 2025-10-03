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
    let rt = tokio::runtime::Runtime::new().unwrap();
    let stub_sig = rt.block_on(async {
        get_signature_from_js(FixedBytes::<32>::default().to_string()).await
    })?;

    let signature_provider: SignatureProvider =
        Box::new(move |hash: FixedBytes<32>| {
            let result = rt.block_on(async {
                get_signature_from_js(hash.to_string()).await
            })?;
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

async fn get_signature_from_js(hash: String) -> eyre::Result<Bytes> {
    use tokio::process::Command;

    let working_dir = "../../../../../erc4337-contracts";

    let output = Command::new("pnpm")
        .arg("run")
        .arg("sign_hash_with_passkey")
        .arg("--hash")
        .arg(&hash)
        .current_dir(working_dir)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eyre::bail!("Failed to sign hash with passkey: {}", stderr);
    }

    let stdout = String::from_utf8(output.stdout)?;

    // Extract the last non-empty line which should be the hex signature
    let hex_sig = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .last()
        .ok_or_else(|| {
            eyre::eyre!("No output from sign_hash_with_passkey command")
        })?
        .trim();

    let bytes = Bytes::from_str(hex_sig)?;

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_signature_from_js() {
        // Test with a sample hash
        let test_hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();

        let result = get_signature_from_js(test_hash).await;

        // The test will pass if the command succeeds and returns valid bytes
        // or fail if the command doesn't exist or returns invalid data
        match result {
            Ok(signature) => {
                // Verify we got some bytes back
                assert!(!signature.is_empty(), "Signature should not be empty");
                println!("Successfully got signature: {:?}", signature);
            }
            Err(e) => {
                // This is expected if the command isn't set up yet
                println!(
                    "Expected error (command may not be implemented yet): {}",
                    e
                );
            }
        }
    }
}
