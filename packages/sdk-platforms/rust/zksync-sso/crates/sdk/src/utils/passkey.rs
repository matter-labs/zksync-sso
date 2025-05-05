use crate::client::passkey::actions::passkey::AuthenticatorAssertionResponseJSON;
use crate::config::contracts::PasskeyContracts;
use base64::Engine;
use eyre::Result;

pub mod normalize_s;
pub mod passkey;
pub mod passkey_signature_from_public_key;
pub mod signature_encoding;
pub mod unwrap_signature;

pub fn passkey_hash_signature_response_format(
    passkey_response: &AuthenticatorAssertionResponseJSON,
    contracts: &PasskeyContracts,
) -> Result<String> {
    let full_formatted_sig = passkey_hash_signature_response_format_bytes(
        passkey_response,
        contracts,
    )?;

    let full_formatted_sig_hex = hex::encode(full_formatted_sig);
    println!("XDB - full_formatted_sig_hex: {:?}", full_formatted_sig_hex);

    Ok(full_formatted_sig_hex)
}

pub fn passkey_hash_signature_response_format_bytes(
    passkey_response: &AuthenticatorAssertionResponseJSON,
    contracts: &PasskeyContracts,
) -> Result<Vec<u8>> {
    println!(
        "XDB - utils::passkey::passkey_hash_signature_response_format - passkey_response: {:?}",
        passkey_response
    );
    println!(
        "XDB - utils::passkey::passkey_hash_signature_response_format - contracts: {:?}",
        contracts
    );

    let passkey_id = passkey_response.credential_id.clone();

    println!(
        "XDB - Decoding auth_data from: {}",
        passkey_response.authenticator_data
    );
    let auth_data = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&passkey_response.authenticator_data)?;
    println!("XDB - Decoded auth_data: {:?}", auth_data);

    println!(
        "XDB - Decoding client_data_json from: {}",
        passkey_response.client_data_json
    );
    let client_data_json = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&passkey_response.client_data_json)?;
    println!("XDB - Decoded client_data_json: {:?}", client_data_json);

    println!("XDB - Decoding signature from: {}", passkey_response.signature);
    let signature = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&passkey_response.signature)?;
    println!("XDB - Decoded signature: {:?}", signature);

    println!("XDB - Unwrapping EC2 signature");
    let unwrapped_sig = unwrap_signature::unwrap_ec2_signature(&signature)?;
    println!(
        "XDB - Unwrapped signature: r={:?}, s={:?}",
        unwrapped_sig.r, unwrapped_sig.s
    );

    let encoded_fat_signature = signature_encoding::encode_fat_signature(
        auth_data,
        client_data_json,
        unwrapped_sig,
        passkey_id,
    )?;
    println!("XDB - encoded_fat_signature: {:?}", encoded_fat_signature);

    let full_formatted_sig = signature_encoding::encode_full_signature(
        encoded_fat_signature,
        contracts,
    )?;
    println!("XDB - full_formatted_sig: {:?}", full_formatted_sig);

    Ok(full_formatted_sig)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn test_passkey_hash_signature_response_format_matches_typescript()
    -> Result<()> {
        let passkey_response = AuthenticatorAssertionResponseJSON {
            credential_id: "CredentialId".to_string(),
            authenticator_data: "08tFjuLOhgB6vt7kcKBTmkNjX9Yu4Wdm0LLy_MUj0v0dAAAAAA".to_string(),
            client_data_json: "eyJ0eXBlIjoid2ViYXV0aG4uZ2V0IiwiY2hhbGxlbmdlIjoiODI3bm9uQVlEbXFnN3J2SFJFVURWOFhGZ3RveHhLZVhxdHJMcERram4zbyIsIm9yaWdpbiI6Imh0dHBzOi8vc29vLXNkay1leGFtcGxlLXBhZ2VzLnBhZ2VzLmRldiJ9".to_string(),
            signature: "MEUCIB5r05hwBHWRD7ZjifF39tSuw5Iw4gwp8BlFfAhn4wd4AiEA1u19vH590rl-FWWuwCVSF6bDsDRG4WI9Uu_y3pKqxV8".to_string(),
            user_handle: None,
        };

        let contracts = PasskeyContracts {
            account_factory: address!(
                "0000000000000000000000000000000000000000"
            ),
            passkey: address!("1234567890123456789012345678901234567890"),
            session: address!("0000000000000000000000000000000000000000"),
            account_paymaster: address!(
                "0000000000000000000000000000000000000000"
            ),
            recovery: address!("0000000000000000000000000000000000000000"),
        };

        let result = passkey_hash_signature_response_format(
            &passkey_response,
            &contracts,
        )?;

        assert_eq!(
            format!("0x{}", result),
            "0x000000000000000000000000000000000000000000000000000000000000006000000000000000000000000012345678901234567890123456789012345678900000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001001e6bd398700475910fb66389f177f6d4aec39230e20c29f019457c0867e307782912824281822d4781ea9a513fdaade816234a7960363c47a0c9d7e469b85ff200000000000000000000000000000000000000000000000000000000000001c00000000000000000000000000000000000000000000000000000000000000025d3cb458ee2ce86007abedee470a0539a43635fd62ee16766d0b2f2fcc523d2fd1d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000847b2274797065223a22776562617574686e2e676574222c226368616c6c656e6765223a223832376e6f6e4159446d716737727648524555445638584667746f78784b65587174724c70446b6a6e336f222c226f726967696e223a2268747470733a2f2f736f6f2d73646b2d6578616d706c652d70616765732e70616765732e646576227d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000090ab79d7a7b626a521d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000"
        );

        Ok(())
    }
}
