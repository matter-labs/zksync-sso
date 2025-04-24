use crate::utils::passkey::passkey::simplewebauthn::{
    _start_authentication, _verify_authentication_response, AuthenticatorInfo,
    CloneablePublicKeyCredentialRequestOptions,
    VerifyAuthenticationResponseArgs,
};
use base64_url::encode as base64_encode_url;
use coset::iana;
use eyre::eyre;
use passkey::types::webauthn::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestPasskeyAuthenticationArgs {
    pub challenge: [u8; 32],
    pub credential_public_key: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rp_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPasskeyAuthenticationResponse {
    pub passkey_authentication_response: AuthenticationResponseJSON,
    pub passkey_authentication_options: PublicKeyCredentialRequestOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePasskeyAuthenticationOptionsArgs {
    pub challenge: [u8; 32],
    pub rp_id: String,
}

fn _generate_passkey_authentication_options(
    args: GeneratePasskeyAuthenticationOptionsArgs,
) -> PublicKeyCredentialRequestOptions {
    PublicKeyCredentialRequestOptions {
        challenge: args.challenge.to_vec().into(),
        timeout: None,
        rp_id: Some(args.rp_id.clone()),
        allow_credentials: None,
        user_verification: UserVerificationRequirement::Discouraged,
        hints: None,
        attestation: AttestationConveyancePreference::None,
        attestation_formats: None,
        extensions: None,
    }
}

async fn _request_passkey_authentication(
    args: RequestPasskeyAuthenticationArgs,
) -> eyre::Result<RequestPasskeyAuthenticationResponse> {
    let rp_id = args.rp_id.clone().ok_or_else(|| eyre!("RP ID is required"))?;

    let passkey_authentication_options =
        _generate_passkey_authentication_options(
            GeneratePasskeyAuthenticationOptionsArgs {
                challenge: args.challenge,
                rp_id: rp_id.clone(),
            },
        );

    let cloneable_public_key_credential_request_options: CloneablePublicKeyCredentialRequestOptions =
        CloneablePublicKeyCredentialRequestOptions {
            challenge: passkey_authentication_options.challenge.clone(),
            rp_id: passkey_authentication_options.rp_id.clone(),
            ..Default::default()
        };

    let authentication_response =
        _start_authentication(cloneable_public_key_credential_request_options)
            .await?;

    let json_response = AuthenticationResponseJSON {
        id: base64_encode_url(&authentication_response.raw_id.to_vec()),
        raw_id: base64_encode_url(&authentication_response.raw_id.to_vec()),
        response: AuthenticatorAssertionResponseJSON {
            credential_id: base64_encode_url(
                &authentication_response.raw_id.to_vec(),
            ),
            client_data_json: base64_encode_url(
                &authentication_response.response.client_data_json.to_vec(),
            ),
            authenticator_data: base64_encode_url(
                &authentication_response.response.authenticator_data.to_vec(),
            ),
            signature: base64_encode_url(
                &authentication_response.response.signature.to_vec(),
            ),
            user_handle: authentication_response
                .response
                .user_handle
                .map(|h| base64_encode_url(&h.to_vec())),
        },
        authenticator_attachment: authentication_response
            .authenticator_attachment,
        client_extension_results: AuthenticationExtensionsClientOutputs {
            app_id: None,
            hmac_create_secret: None,
        },
        type_: authentication_response.ty,
    };

    let verification =
        _verify_authentication_response(VerifyAuthenticationResponseArgs {
            response: json_response.clone(),
            expected_challenge: passkey_authentication_options
                .challenge
                .clone(),
            expected_origin: args
                .origin
                .ok_or_else(|| eyre!("Origin is required"))?,
            expected_rp_id: rp_id,
            authenticator: AuthenticatorInfo {
                credential_public_key: args.credential_public_key,
                credential_id: json_response.id.clone(),
                counter: 0,
            },
        })
        .await?;

    if !verification.verified || verification.authentication_info.is_none() {
        return Err(eyre::eyre!("Passkey validation failed"));
    }

    Ok(RequestPasskeyAuthenticationResponse {
        passkey_authentication_response: json_response,
        passkey_authentication_options,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePasskeyRegistrationOptionsArgs {
    pub user_name: String,
    pub user_display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rp_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rp_id: Option<String>,
}

pub async fn generate_passkey_registration_options(
    args: GeneratePasskeyRegistrationOptionsArgs,
) -> eyre::Result<CredentialCreationOptions> {
    let rp_name =
        args.rp_name.unwrap_or_else(|| "default.example.com".to_string());

    let rp_id = args.rp_id.unwrap_or_else(|| "default.example.com".to_string());

    let user_id = rand::random::<[u8; 32]>();

    let options = CredentialCreationOptions {
        public_key: PublicKeyCredentialCreationOptions {
            rp: PublicKeyCredentialRpEntity { id: Some(rp_id), name: rp_name },
            user: PublicKeyCredentialUserEntity {
                id: user_id.to_vec().into(),
                name: args.user_name,
                display_name: args.user_display_name,
            },
            challenge: rand::random::<[u8; 32]>().to_vec().into(),
            pub_key_cred_params: vec![PublicKeyCredentialParameters {
                ty: PublicKeyCredentialType::PublicKey,
                alg: iana::Algorithm::ES256,
            }],
            timeout: None,
            exclude_credentials: None,
            authenticator_selection: Some(AuthenticatorSelectionCriteria {
                authenticator_attachment: None,
                resident_key: Some(ResidentKeyRequirement::Required),
                user_verification: UserVerificationRequirement::Discouraged,
                require_resident_key: true,
            }),
            hints: None,
            attestation: AttestationConveyancePreference::Direct,
            attestation_formats: None,
            extensions: None,
        },
    };

    Ok(options)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationResponseJSON {
    pub id: String,
    pub raw_id: String,
    pub response: AuthenticatorAssertionResponseJSON,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticator_attachment: Option<AuthenticatorAttachment>,
    pub client_extension_results: AuthenticationExtensionsClientOutputs,
    #[serde(rename = "type")]
    pub type_: PublicKeyCredentialType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorAssertionResponseJSON {
    pub credential_id: String,
    pub client_data_json: String,
    pub authenticator_data: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_handle: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationExtensionsClientOutputs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hmac_create_secret: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_passkey_registration_options() {
        let args = GeneratePasskeyRegistrationOptionsArgs {
            user_name: "test.user@example.com".to_string(),
            user_display_name: "Test User".to_string(),
            rp_name: Some("example.com".to_string()),
            rp_id: Some("example.com".to_string()),
        };

        let result = generate_passkey_registration_options(args).await;
        assert!(result.is_ok());

        let options = result.unwrap();
        assert_eq!(options.public_key.rp.name, "example.com");
        assert!(options.public_key.rp.id.is_some());
        assert_eq!(options.public_key.rp.id.unwrap(), "example.com");
        assert_eq!(options.public_key.user.name, "test.user@example.com");
        assert_eq!(options.public_key.user.display_name, "Test User");
        assert_eq!(
            options.public_key.attestation,
            AttestationConveyancePreference::Direct
        );
        let auth_selection =
            options.public_key.authenticator_selection.unwrap();
        assert_eq!(
            auth_selection.resident_key,
            Some(ResidentKeyRequirement::Required)
        );
        assert_eq!(
            auth_selection.user_verification,
            UserVerificationRequirement::Discouraged
        );
    }

    #[tokio::test]
    async fn test_request_passkey_authentication() -> eyre::Result<()> {
        let args = RequestPasskeyAuthenticationArgs {
            challenge: [1u8; 32],
            credential_public_key: vec![2u8; 32],
            rp_id: Some("example.com".to_string()),
            origin: Some("https://example.com".to_string()),
        };

        let result = _request_passkey_authentication(args).await?;

        assert_eq!(
            result.passkey_authentication_options.challenge.to_vec(),
            vec![1u8; 32]
        );
        assert_eq!(
            result.passkey_authentication_options.rp_id.unwrap(),
            "example.com"
        );
        assert_eq!(
            result.passkey_authentication_options.user_verification,
            UserVerificationRequirement::Discouraged
        );

        assert!(!result.passkey_authentication_response.id.is_empty());
        assert!(!result.passkey_authentication_response.raw_id.is_empty());
        assert!(
            !result
                .passkey_authentication_response
                .response
                .signature
                .is_empty()
        );

        Ok(())
    }
}
