use crate::{
    client::passkey::actions::passkey::AuthenticatorAssertionResponseJSON,
    config::Config,
    utils::passkey::passkey_hash_signature_response_format_bytes,
};
use eyre::Result;

pub mod apple_authorization_assertion;

pub(crate) fn hash_signature_response_format(
    signature_response: Vec<u8>,
    config: &Config,
) -> Result<Vec<u8>> {
    println!(
        "XDB - api::account::sign::hash_signature_response_format signature_response: {:?}",
        signature_response
    );

    let passkey_response = decode_signature_response(&signature_response)?;

    let result = passkey_hash_signature_response_format_bytes(
        &passkey_response,
        &config.contracts,
    )?;

    println!(
        "XDB - api::account::sign::hash_signature_response_format result: {:?}",
        result
    );

    Ok(result)
}

fn decode_signature_response(
    signature_response: &[u8],
) -> Result<AuthenticatorAssertionResponseJSON> {
    println!(
        "XDB - api::account::sign::decode_signature_response signature_response: {:?}",
        signature_response
    );

    println!(
        "XDB - api::account::sign::decode_signature_response signature_response: {:?}",
        String::from_utf8(signature_response.to_vec()).unwrap()
    );

    let assertion: apple_authorization_assertion::AuthorizationPlatformPublicKeyCredentialAssertion =
        serde_json::from_slice(signature_response)?;

    println!(
        "XDB - api::account::sign::decode_signature_response assertion: {:?}",
        assertion
    );

    println!(
        "XDB - api::account::sign::decode_signature_response assertion.signature: {:?}",
        assertion.signature
    );

    let client_data_json = base64_url::encode(&assertion.raw_client_data_json);
    let authenticator_data =
        base64_url::encode(&assertion.raw_authenticator_data);
    let signature = base64_url::encode(&assertion.signature);

    let response = AuthenticatorAssertionResponseJSON {
        client_data_json,
        authenticator_data,
        signature,
        user_handle: None,
    };

    println!(
        "XDB - api::account::sign::decode_signature_response response: {:?}",
        response
    );

    Ok(response)
}
