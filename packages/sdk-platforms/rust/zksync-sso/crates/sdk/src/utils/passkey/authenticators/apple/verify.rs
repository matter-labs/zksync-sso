use crate::utils::passkey::authenticators::verify::ValidatedPasskey;
use log::error;
use passkey::client::apple::{
    ApplePasskeyRegistration, ValidatedPasskey as PasskeyValidated,
};

impl From<PasskeyValidated> for ValidatedPasskey {
    fn from(validated: PasskeyValidated) -> Self {
        Self {
            public_key: validated.public_key,
            credential_id: validated.credential_id,
            cose_key_cbor: validated.cose_key_cbor,
        }
    }
}

pub(crate) async fn verify_registration(
    raw_attestation_object: &[u8],
    raw_client_data_json: &[u8],
    credential_id: &[u8],
    expected_origin: &str,
) -> eyre::Result<ValidatedPasskey> {
    let registration = ApplePasskeyRegistration {
        raw_attestation_object: raw_attestation_object.to_vec(),
        raw_client_data_json: raw_client_data_json.to_vec(),
        credential_id: credential_id.to_vec(),
    };

    let client_data: serde_json::Value =
        serde_json::from_slice(raw_client_data_json)?;
    let challenge = base64_url::decode(
        client_data["challenge"]
            .as_str()
            .ok_or_else(|| eyre::eyre!("Missing challenge"))?,
    )?;

    let validated = registration
        .validate(&challenge, expected_origin)
        .map_err(|e| {
            error!("Validation failed: {:?}", e);
            eyre::eyre!("Passkey validation failed: {:?}", e)
        })
        .map(Into::into)?;

    Ok(validated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use passkey::{
        authenticator::{
            Authenticator, MemoryStore, UserCheck, UserValidationMethod,
        },
        client::{Client, DefaultClientData},
        types::{Passkey, crypto::sha256, ctap2::*, webauthn::*},
    };
    use url::Url;

    struct MyUserValidationMethod {}
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

    #[tokio::test]
    async fn test_verify_registration() -> eyre::Result<()> {
        let challenge = sha256(b"test challenge");
        let rp_id = "future.1password.com";
        let origin = Url::parse(&format!("https://{}", rp_id)).unwrap();

        let user_validation_method = MyUserValidationMethod {};

        let mut client = Client::new(Authenticator::new(
            passkey_types::ctap2::Aaguid::new_empty(),
            MemoryStore::new(),
            user_validation_method,
        ));

        let options = CredentialCreationOptions {
            public_key: PublicKeyCredentialCreationOptions {
                rp: PublicKeyCredentialRpEntity {
                    id: None,
                    name: rp_id.into(),
                },
                user: PublicKeyCredentialUserEntity {
                    id: passkey_types::rand::random_vec(32).into(),
                    display_name: "Johnny Passkey".into(),
                    name: "jpasskey@example.org".into(),
                },
                challenge: challenge.to_vec().into(),
                pub_key_cred_params: vec![PublicKeyCredentialParameters {
                    ty: PublicKeyCredentialType::PublicKey,
                    alg: coset::iana::Algorithm::ES256,
                }],
                timeout: None,
                exclude_credentials: None,
                authenticator_selection: None,
                attestation: AttestationConveyancePreference::None,
                attestation_formats: None,
                extensions: None,
                hints: None,
            },
        };

        let cred = client
            .register(&origin, options, DefaultClientData)
            .await
            .expect("failed to register");

        let result = verify_registration(
            &cred.response.attestation_object,
            &cred.response.client_data_json,
            &cred.raw_id,
            rp_id,
        )
        .await?;

        assert!(!result.public_key.is_empty());

        Ok(())
    }
}
