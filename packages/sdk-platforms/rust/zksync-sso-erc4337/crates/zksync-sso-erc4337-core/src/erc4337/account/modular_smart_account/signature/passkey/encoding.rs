use crate::utils::passkey::unwrap_signature::UnwrappedSignature;
use alloy::{dyn_abi::SolType, primitives::Address, sol};
use base64::Engine;
use eyre::Result;

type FatSignature = sol! { tuple(bytes, bytes, bytes32[2], bytes) };

pub fn encode_fat_signature(
    auth_data: Vec<u8>,
    client_data_json: Vec<u8>,
    unwrapped_sig: UnwrappedSignature,
    passkey_id: String,
) -> Result<Vec<u8>> {
    let passkey_id_bytes = base64_url_to_uint8_array(passkey_id, true)?;
    let encoded_fat_signature = FatSignature::abi_encode_params(&(
        auth_data,
        client_data_json,
        [
            <[u8; 32]>::try_from(unwrapped_sig.r.as_slice()).unwrap(),
            <[u8; 32]>::try_from(unwrapped_sig.s.as_slice()).unwrap(),
        ],
        passkey_id_bytes,
    ));
    Ok(encoded_fat_signature)
}

pub fn encode_full_signature(
    encoded_fat_signature: Vec<u8>,
    webauthn_validator: Address,
) -> Result<Vec<u8>> {
    let encoded_full_signature = [
        webauthn_validator.to_vec().as_slice(),
        encoded_fat_signature.as_slice(),
    ]
    .concat()
    .to_vec();
    Ok(encoded_full_signature)
}

pub(crate) fn base64_url_to_uint8_array(
    base64url_string: String,
    is_url: bool,
) -> Result<Vec<u8>> {
    let result = if is_url {
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(base64url_string)
    } else {
        base64::engine::general_purpose::STANDARD.decode(base64url_string)
    };
    result.map_err(|e| eyre::eyre!("Failed to decode base64: {}", e))
}
