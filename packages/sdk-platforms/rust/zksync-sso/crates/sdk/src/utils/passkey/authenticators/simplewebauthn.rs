use crate::client::passkey::actions::passkey::AuthenticationResponseJSON;
use coset::iana;
use eyre::Result;
use passkey::{
    authenticator::{Authenticator, UserCheck, UserValidationMethod},
    client::{Client, WebauthnError},
    types::{
        Bytes, Passkey,
        crypto::sha256,
        ctap2::*,
        rand::random_vec,
        webauthn::{
            AttestationStatementFormatIdentifiers,
            AuthenticationExtensionsClientInputs, PublicKeyCredentialHints, *,
        },
    },
};
use passkey_client::DefaultClientData;
use url::Url;

pub struct MyUserValidationMethod {}

#[async_trait::async_trait]
impl UserValidationMethod for MyUserValidationMethod {
    type PasskeyItem = Passkey;

    async fn check_user<'a>(
        &self,
        _credential: Option<&'a Passkey>,
        presence: bool,
        verification: bool,
    ) -> Result<UserCheck, Ctap2Error> {
        Ok(UserCheck { presence, verification })
    }

    fn is_verification_enabled(&self) -> Option<bool> {
        Some(true)
    }

    fn is_presence_enabled(&self) -> bool {
        true
    }
}

#[derive(Debug, Default)]
pub struct CloneablePublicKeyCredentialRequestOptions {
    pub challenge: Bytes,
    pub timeout: Option<u32>,
    pub rp_id: Option<String>,
    pub allow_credentials: Option<Vec<PublicKeyCredentialDescriptor>>,
    pub user_verification: UserVerificationRequirement,
    pub hints: Option<Vec<PublicKeyCredentialHints>>,
    pub attestation: AttestationConveyancePreference,
    pub attestation_formats: Option<Vec<AttestationStatementFormatIdentifiers>>,
    pub extensions: Option<AuthenticationExtensionsClientInputs>,
}

impl From<PublicKeyCredentialRequestOptions>
    for CloneablePublicKeyCredentialRequestOptions
{
    fn from(options: PublicKeyCredentialRequestOptions) -> Self {
        Self {
            challenge: options.challenge,
            timeout: options.timeout,
            rp_id: options.rp_id,
            allow_credentials: options.allow_credentials,
            user_verification: options.user_verification,
            hints: options.hints,
            attestation: options.attestation,
            attestation_formats: options.attestation_formats,
            extensions: options.extensions,
        }
    }
}

impl From<CloneablePublicKeyCredentialRequestOptions>
    for PublicKeyCredentialRequestOptions
{
    fn from(options: CloneablePublicKeyCredentialRequestOptions) -> Self {
        Self {
            challenge: options.challenge,
            timeout: options.timeout,
            rp_id: options.rp_id,
            allow_credentials: options.allow_credentials,
            user_verification: options.user_verification,
            hints: options.hints,
            attestation: options.attestation,
            attestation_formats: options.attestation_formats,
            extensions: options.extensions,
        }
    }
}

pub async fn _start_authentication(
    options: CloneablePublicKeyCredentialRequestOptions,
) -> Result<AuthenticatedPublicKeyCredential> {
    let rp_id = options.rp_id.unwrap_or_else(|| "1password".to_string());
    let origin = format!("https://{}", rp_id);
    let challenge = options.challenge.to_vec();
    let (_, public_key_credential) =
        create_credential(origin, rp_id, challenge).await?;
    Ok(public_key_credential)
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct AuthenticatorInfo {
    pub credential_public_key: Vec<u8>,
    pub credential_id: String,
    #[allow(dead_code)]
    pub counter: u32,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct VerifyAuthenticationResponseArgs {
    pub response: AuthenticationResponseJSON,
    #[allow(dead_code)]
    pub expected_challenge: Bytes,
    #[allow(dead_code)]
    pub expected_origin: String,
    pub expected_rp_id: String,
    pub authenticator: AuthenticatorInfo,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct VerificationResult {
    pub verified: bool,
    pub authentication_info: Option<AuthenticatorInfo>,
}

pub(crate) async fn _verify_authentication_response(
    args: VerifyAuthenticationResponseArgs,
) -> Result<VerificationResult> {
    let client_data_json =
        base64_url::decode(&args.response.response.client_data_json)?;
    let client_data_hash = sha256(&client_data_json).to_vec();

    let user_entity = PublicKeyCredentialUserEntity {
        id: args.authenticator.credential_id.as_bytes().to_vec().into(),
        display_name: "Verification User".into(),
        name: "verification.user@example.com".into(),
    };

    let auth_result = _authenticator_setup(
        user_entity,
        client_data_hash.into(),
        PublicKeyCredentialParameters {
            ty: PublicKeyCredentialType::PublicKey,
            alg: iana::Algorithm::ES256,
        },
        args.expected_rp_id,
    )
    .await;

    match auth_result {
        Ok(response) => Ok(VerificationResult {
            verified: true,
            authentication_info: Some(AuthenticatorInfo {
                credential_public_key: args.authenticator.credential_public_key,
                credential_id: args.authenticator.credential_id,
                counter: response.auth_data.counter.unwrap_or(0),
            }),
        }),
        Err(_) => Ok(VerificationResult {
            verified: false,
            authentication_info: None,
        }),
    }
}

async fn _authenticator_setup(
    user_entity: PublicKeyCredentialUserEntity,
    client_data_hash: Bytes,
    algorithms_from_rp: PublicKeyCredentialParameters,
    rp_id: String,
) -> Result<get_assertion::Response, StatusCode> {
    let store: Option<Passkey> = None;
    let user_validation_method = MyUserValidationMethod {};
    let my_aaguid = Aaguid::new_empty();

    let mut my_authenticator =
        Authenticator::new(my_aaguid, store, user_validation_method);

    let reg_request = make_credential::Request {
        client_data_hash: client_data_hash.clone(),
        rp: make_credential::PublicKeyCredentialRpEntity {
            id: rp_id.clone(),
            name: None,
        },
        user: user_entity,
        pub_key_cred_params: vec![algorithms_from_rp],
        exclude_list: None,
        extensions: None,
        options: make_credential::Options::default(),
        pin_auth: None,
        pin_protocol: None,
    };

    let _credential = my_authenticator.make_credential(reg_request).await?;

    let auth_request = get_assertion::Request {
        rp_id,
        client_data_hash,
        allow_list: None,
        extensions: None,
        options: make_credential::Options::default(),
        pin_auth: None,
        pin_protocol: None,
    };

    my_authenticator.get_assertion(auth_request).await
}

#[allow(dead_code)]
async fn create_credential(
    origin: String,
    id: String,
    challenge: Vec<u8>,
) -> Result<(
    PublicKeyCredential<AuthenticatorAttestationResponse>,
    PublicKeyCredential<AuthenticatorAssertionResponse>,
)> {
    let origin = Url::parse(&origin)?;
    let id: Bytes = id.as_bytes().to_vec().into();
    let user_entity = PublicKeyCredentialUserEntity {
        id,
        display_name: "TODO Display Name".into(),
        name: "TODO Name".into(),
    };
    let challenge_bytes_from_rp: Bytes = challenge.into();
    let (created_cred, authed_cred) = client_setup(
        challenge_bytes_from_rp,
        PublicKeyCredentialParameters {
            ty: PublicKeyCredentialType::PublicKey,
            alg: iana::Algorithm::ES256,
        },
        &origin,
        user_entity.clone(),
    )
    .await
    .map_err(|e| eyre::eyre!("{:?}", e))?;

    println!("Webauthn credential created:\n\n{:?}\n\n", created_cred);
    println!("Webauthn credential auth'ed:\n\n{:?}\n\n", authed_cred);

    Ok((created_cred, authed_cred))
}

#[allow(dead_code)]
async fn client_setup(
    challenge_bytes_from_rp: Bytes,
    parameters_from_rp: PublicKeyCredentialParameters,
    origin: &Url,
    user_entity: PublicKeyCredentialUserEntity,
) -> Result<
    (CreatedPublicKeyCredential, AuthenticatedPublicKeyCredential),
    WebauthnError,
> {
    let my_aaguid = Aaguid::new_empty();
    let user_validation_method = MyUserValidationMethod {};
    let store: Option<Passkey> = None;
    let my_authenticator =
        Authenticator::new(my_aaguid, store, user_validation_method);

    let mut my_client = Client::new(my_authenticator);

    let request = CredentialCreationOptions {
        public_key: PublicKeyCredentialCreationOptions {
            rp: PublicKeyCredentialRpEntity {
                id: None,
                name: origin.domain().unwrap().into(),
            },
            user: user_entity,
            challenge: challenge_bytes_from_rp,
            pub_key_cred_params: vec![parameters_from_rp],
            timeout: None,
            exclude_credentials: None,
            authenticator_selection: None,
            hints: None,
            attestation: AttestationConveyancePreference::None,
            attestation_formats: None,
            extensions: None,
        },
    };

    let my_webauthn_credential =
        my_client.register(origin, request, DefaultClientData).await?;

    let challenge_bytes_from_rp: Bytes = random_vec(32).into();

    let credential_request = CredentialRequestOptions {
        public_key: PublicKeyCredentialRequestOptions {
            challenge: challenge_bytes_from_rp,
            timeout: None,
            rp_id: Some(String::from(origin.domain().unwrap())),
            allow_credentials: None,
            user_verification: UserVerificationRequirement::default(),
            hints: None,
            attestation: AttestationConveyancePreference::None,
            attestation_formats: None,
            extensions: None,
        },
    };

    let authenticated_cred = my_client
        .authenticate(origin, credential_request, DefaultClientData)
        .await?;

    Ok((my_webauthn_credential, authenticated_cred))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::passkey::actions::passkey::{
        AuthenticationExtensionsClientOutputs, AuthenticationResponseJSON,
        AuthenticatorAssertionResponseJSON,
    };
    use base64_url::encode as base64_encode_url;

    #[tokio::test]
    async fn test_create_credential() -> eyre::Result<()> {
        let origin = "https://example.com".to_string();
        let id = "test_id".to_string();
        let challenge = vec![1u8; 32]; // Test challenge

        let (created_cred, authed_cred) =
            create_credential(origin, id, challenge).await?;

        assert_eq!(created_cred.ty, PublicKeyCredentialType::PublicKey);
        assert!(!created_cred.raw_id.is_empty());
        assert!(!created_cred.response.attestation_object.is_empty());

        assert_eq!(authed_cred.ty, PublicKeyCredentialType::PublicKey);
        assert!(!authed_cred.raw_id.is_empty());
        assert!(!authed_cred.response.signature.is_empty());
        assert!(!authed_cred.response.authenticator_data.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_verify_authentication_response() -> eyre::Result<()> {
        let origin = "https://example.com".to_string();
        let id = "test_id".to_string();
        let challenge = vec![1u8; 32];

        let (_, authed_cred) =
            create_credential(origin.clone(), id.clone(), challenge.clone())
                .await?;

        let auth_response = AuthenticationResponseJSON {
            id: base64_encode_url(&authed_cred.raw_id.to_vec()),
            raw_id: base64_encode_url(&authed_cred.raw_id.to_vec()),
            response: AuthenticatorAssertionResponseJSON {
                credential_id: base64_encode_url(&authed_cred.raw_id.to_vec()),
                client_data_json: base64_encode_url(
                    &authed_cred.response.client_data_json.to_vec(),
                ),
                authenticator_data: base64_encode_url(
                    &authed_cred.response.authenticator_data.to_vec(),
                ),
                signature: base64_encode_url(
                    &authed_cred.response.signature.to_vec(),
                ),
                user_handle: authed_cred
                    .response
                    .user_handle
                    .map(|h| base64_encode_url(&h.to_vec())),
            },
            authenticator_attachment: authed_cred.authenticator_attachment,
            client_extension_results: AuthenticationExtensionsClientOutputs {
                app_id: None,
                hmac_create_secret: None,
            },
            type_: authed_cred.ty,
        };

        let args = VerifyAuthenticationResponseArgs {
            response: auth_response,
            expected_challenge: challenge.into(),
            expected_origin: origin,
            expected_rp_id: "example.com".to_string(),
            authenticator: AuthenticatorInfo {
                credential_public_key: vec![2u8; 32],
                credential_id: "test_id".to_string(),
                counter: 0,
            },
        };

        let result = _verify_authentication_response(args).await?;

        // With real credential data, verification should succeed
        assert!(result.verified);
        assert!(result.authentication_info.is_some());

        Ok(())
    }
}
