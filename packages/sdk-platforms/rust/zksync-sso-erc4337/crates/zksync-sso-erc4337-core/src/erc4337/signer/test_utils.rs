use crate::erc4337::signer::{SignatureProvider, Signer};
use alloy::primitives::{Bytes, FixedBytes};
use eyre;
use std::{pin::Pin, process::Command, str::FromStr, sync::Arc};

pub fn get_signature_from_js(hash: String) -> eyre::Result<Bytes> {
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

    let last_line = stdout
        .lines()
        .rfind(|line| !line.is_empty())
        .ok_or_else(|| eyre::eyre!("No output from sign_hash_with_passkey command"))?;

    let hex_sig = last_line.trim();
    let bytes = Bytes::from_str(hex_sig)?;

    Ok(bytes)
}

pub fn create_test_webauthn_js_signer() -> Signer {
    let stub_sig =
        get_signature_from_js(FixedBytes::<32>::default().to_string())
            .expect("Failed to create stub signature for WebAuthn signer");

    let signature_provider: SignatureProvider =
        Arc::new(
            move |hash: FixedBytes<32>| -> Pin<
                Box<dyn Future<Output = eyre::Result<Bytes>> + Send>,
            > {
                Box::pin(async move { get_signature_from_js(hash.to_string()) })
                    as Pin<Box<dyn Future<Output = eyre::Result<Bytes>> + Send>>
            },
        );

    Signer { stub_signature: stub_sig, provider: signature_provider }
}
