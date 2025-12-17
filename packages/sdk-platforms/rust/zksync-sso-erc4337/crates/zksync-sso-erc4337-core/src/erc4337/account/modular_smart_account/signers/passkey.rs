use alloy::primitives::{Address, Bytes, FixedBytes};
use alloy::sol_types::SolType;

pub fn stub_signature_passkey(validator: Address) -> eyre::Result<Bytes> {
    // Create a more realistic WebAuthn stub signature for gas estimation
    // Format: validator_address (20 bytes) + abi_encoded(authenticatorData, clientDataJSON, rs, credentialId)
    
    // Minimal valid authenticator data (37 bytes minimum)
    let authenticator_data = vec![
        0x49, 0x96, 0x0d, 0xe5, 0x88, 0x0e, 0x8c, 0x68, 0x74, 0x34, 
        0x17, 0x0f, 0x64, 0x76, 0x60, 0x5b, 0x8f, 0xe4, 0xae, 0xb9,
        0xa2, 0x86, 0x32, 0xc7, 0x99, 0x5c, 0xf3, 0xba, 0x83, 0x1d,
        0x97, 0x63, // rpIdHash (32 bytes)
        0x01, // flags
        0x00, 0x00, 0x00, 0x00, // counter (4 bytes)
    ];
    
    // Minimal client data JSON for gas estimation
    let client_data_json = r#"{"type":"webauthn.get","challenge":"AAAAAAAAAAAAAAAAAAAAAA"}"#.to_string();
    
    // Stub signature components (r and s)
    let r_fixed = FixedBytes::<32>::ZERO;
    let s_fixed = FixedBytes::<32>::ZERO;
    
    // Stub credential ID
    let credential_id = vec![0u8; 16];
    
    // ABI encode the WebAuthn signature: (bytes authenticatorData, string clientDataJSON, bytes32[2] rs, bytes credentialId)
    type SignatureParams = (
        alloy::sol_types::sol_data::Bytes,
        alloy::sol_types::sol_data::String,
        alloy::sol_types::sol_data::FixedArray<
            alloy::sol_types::sol_data::FixedBytes<32>,
            2,
        >,
        alloy::sol_types::sol_data::Bytes,
    );

    let params = (
        authenticator_data,
        client_data_json,
        [r_fixed, s_fixed],
        credential_id,
    );

    let encoded_signature = SignatureParams::abi_encode_params(&params);
    
    // Prepend validator address (20 bytes) to the encoded signature
    let mut result = Vec::with_capacity(20 + encoded_signature.len());
    result.extend_from_slice(validator.as_slice());
    result.extend_from_slice(&encoded_signature);
    
    Ok(Bytes::from(result))
}
