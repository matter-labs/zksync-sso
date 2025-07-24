use crate::client::passkey::actions::deploy::CredentialDetails;

pub(crate) fn get_mock_credential_details() -> CredentialDetails {
    let id = "unique-base64encoded-string".to_string();
    let public_key = vec![
        165, 1, 2, 3, 38, 32, 1, 33, 88, 32, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 34, 88,
        32, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    ];
    CredentialDetails { id, public_key }
}
