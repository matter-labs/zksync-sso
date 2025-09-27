use crate::utils::create_error;
use alloy::primitives::FixedBytes;
use alloy::signers::local::PrivateKeySigner;

pub fn create_private_key_signer(private_key: &str) -> Result<String, String> {
    let key_bytes = alloy::primitives::hex::decode(private_key.trim_start_matches("0x"))
        .map_err(|e| create_error(&e.to_string()))?;

    let signer = PrivateKeySigner::from_bytes(
        &FixedBytes::<32>::try_from(key_bytes.as_slice())
            .map_err(|e| create_error(&format!("Invalid key length: {e}")))?,
    )
    .map_err(|e| create_error(&e.to_string()))?;

    Ok(format!(
        "0x{}",
        alloy::primitives::hex::encode(signer.address().as_slice())
    ))
}
