#[cfg(test)]
pub mod tests {
    use crate::{
        api::account::{
            deployment::{DeployedAccountDetails, deploy_account},
            passkey::{passkey_parameters::PasskeyParameters, rp_id::RpId},
        },
        config::Config,
        utils::test_utils::spawn_node_and_deploy_contracts,
    };
    use eyre::Result;
    use passkey::{
        authenticator::{
            Authenticator, CredentialStore, MemoryStore,
            MockUserValidationMethod, UserCheck,
        },
        client::{Client, DefaultClientData, WebauthnError},
        types::{
            Bytes, Passkey,
            ctap2::*,
            webauthn::{
                self, AttestationConveyancePreference,
                CredentialCreationOptions, CredentialRequestOptions,
                PublicKeyCredentialCreationOptions,
                PublicKeyCredentialDescriptor, PublicKeyCredentialParameters,
                PublicKeyCredentialRequestOptions, PublicKeyCredentialRpEntity,
                PublicKeyCredentialType, PublicKeyCredentialUserEntity,
                ResidentKeyRequirement, UserVerificationRequirement,
            },
        },
    };
    use sha2::{Digest, Sha256};
    use url::Url;

    pub struct AuthStack {
        pub client: Client<
            MemoryStore,
            MockUserValidationMethod,
            public_suffix::PublicSuffixList,
        >,
    }

    pub fn get_origin(rp_id: &str) -> Url {
        Url::parse(&format!("https://{rp_id}")).unwrap()
    }

    pub fn create_auth_stack() -> AuthStack {
        let store = MemoryStore::new();
        let mut mock_uv = MockUserValidationMethod::new();
        mock_uv.expect_check_user().returning(|_, _, _| {
            Ok(UserCheck { presence: true, verification: true })
        });
        mock_uv.expect_is_verification_enabled().returning(|| Some(true));
        mock_uv.expect_is_presence_enabled().returning(|| true);

        let client = Client::new(Authenticator::new(
            Aaguid::new_empty(),
            store,
            mock_uv,
        ));
        AuthStack { client }
    }

    pub async fn find_credentials(
        auth_stack: &mut AuthStack,
        rp_id: &str,
        credential_raw_id: Bytes,
    ) -> Result<Passkey, WebauthnError> {
        let ids =
            vec![passkey_types::webauthn::PublicKeyCredentialDescriptor {
                ty: PublicKeyCredentialType::PublicKey,
                id: credential_raw_id.clone(),
                transports: None,
            }];

        let credentials = auth_stack
            .client
            .authenticator()
            .store()
            .find_credentials(Some(&ids), rp_id)
            .await?;
        println!(
            "XDB - find_credentials - Found credentials for ID {credential_raw_id:?}: {credentials:?}"
        );

        if credentials.is_empty() {
            return Err(WebauthnError::CredentialNotFound);
        }

        Ok(credentials.first().unwrap().to_owned())
    }

    pub async fn authenticate_apple_passkey(
        auth_stack: &mut AuthStack,
        rp_id: &str,
        challenge: Vec<u8>,
        credential_raw_id: Vec<u8>,
    ) -> Result<
        webauthn::PublicKeyCredential<webauthn::AuthenticatorAssertionResponse>,
        WebauthnError,
    > {
        let origin = get_origin(rp_id);

        let ids =
            vec![passkey_types::webauthn::PublicKeyCredentialDescriptor {
                ty: PublicKeyCredentialType::PublicKey,
                id: credential_raw_id.into(),
                transports: None,
            }];

        let passkeys = auth_stack
            .client
            .authenticator()
            .store()
            .find_credentials(Some(&ids), rp_id)
            .await?;
        println!(
            "XDB - authenticate_apple_passkey - Available passkeys: {passkeys:?}"
        );

        let saved_ids: Vec<Vec<u8>> =
            passkeys.iter().map(|p| p.credential_id.to_vec()).collect();

        let first_id = saved_ids.first().unwrap().to_owned();

        let ids: Vec<PublicKeyCredentialDescriptor> =
            vec![PublicKeyCredentialDescriptor {
                ty: PublicKeyCredentialType::PublicKey,
                id: first_id.into(),
                transports: None,
            }];

        let options = CredentialRequestOptions {
            public_key: PublicKeyCredentialRequestOptions {
                challenge: challenge.into(),
                timeout: None,
                rp_id: Some(rp_id.to_string()),
                allow_credentials: Some(ids),
                user_verification: UserVerificationRequirement::Required,
                hints: None,
                attestation: AttestationConveyancePreference::None,
                attestation_formats: None,
                extensions: None,
            },
        };

        let auth_response = auth_stack
            .client
            .authenticate(&origin, options, DefaultClientData)
            .await?;
        println!(
            "XDB - authenticate_apple_passkey - Auth response credential ID: {:?}",
            auth_response.id
        );
        Ok(auth_response)
    }

    pub async fn register_credential(
        auth_stack: &mut AuthStack,
        rp_id: &str,
        user_name: &str,
        user_display_name: &str,
        challenge: Vec<u8>,
    ) -> Result<
        webauthn::PublicKeyCredential<
            webauthn::AuthenticatorAttestationResponse,
        >,
        WebauthnError,
    > {
        let origin = get_origin(rp_id);

        let options = CredentialCreationOptions {
            public_key: PublicKeyCredentialCreationOptions {
                rp: PublicKeyCredentialRpEntity {
                    id: Some(rp_id.to_string()),
                    name: rp_id.into(),
                },
                user: PublicKeyCredentialUserEntity {
                    id: passkey_types::rand::random_vec(32).into(),
                    display_name: user_display_name.into(),
                    name: user_name.into(),
                },
                challenge: challenge.into(),
                pub_key_cred_params: vec![PublicKeyCredentialParameters {
                    ty: PublicKeyCredentialType::PublicKey,
                    alg: coset::iana::Algorithm::ES256,
                }],
                timeout: None,
                exclude_credentials: None,
                authenticator_selection: Some(
                    webauthn::AuthenticatorSelectionCriteria {
                        authenticator_attachment: None,
                        require_resident_key: true,
                        user_verification:
                            UserVerificationRequirement::Required,
                        resident_key: Some(ResidentKeyRequirement::Required),
                    },
                ),
                attestation: AttestationConveyancePreference::Direct,
                attestation_formats: None,
                extensions: None,
                hints: None,
            },
        };

        let credential = auth_stack
            .client
            .register(&origin, options, DefaultClientData)
            .await?;
        println!(
            "XDB - register_credential - Registered credential ID: {:?}",
            credential.id
        );
        Ok(credential)
    }

    pub async fn register_apple_passkey(
        auth_stack: &mut AuthStack,
        rp_id: &str,
        user_name: &str,
        user_display_name: &str,
        challenge: Vec<u8>,
    ) -> Result<PasskeyParameters, WebauthnError> {
        let credential = register_credential(
            auth_stack,
            rp_id,
            user_name,
            user_display_name,
            challenge,
        )
        .await?;

        let rp_id = RpId::Apple(rp_id.to_string());

        Ok(PasskeyParameters {
            credential_raw_attestation_object: credential
                .response
                .attestation_object
                .into(),
            credential_raw_client_data_json: credential
                .response
                .client_data_json
                .into(),
            credential_id: credential.id.into(),
            rp_id,
        })
    }

    pub async fn deploy_account_with_apple_passkey(
        config: &Config,
        auth_stack: &mut AuthStack,
        rp_id: &str,
        user_name: &str,
        user_display_name: &str,
        challenge: Vec<u8>,
    ) -> eyre::Result<(
        DeployedAccountDetails,
        webauthn::PublicKeyCredential<
            webauthn::AuthenticatorAttestationResponse,
        >,
    )> {
        let credential = register_credential(
            auth_stack,
            rp_id,
            user_name,
            user_display_name,
            challenge.clone(),
        )
        .await
        .map_err(|e| eyre::eyre!("Error registering Apple passkey: {:?}", e))?;

        let deploy_args = register_apple_passkey(
            auth_stack,
            rp_id,
            user_name,
            user_display_name,
            challenge,
        )
        .await
        .map_err(|e| eyre::eyre!("Error registering Apple passkey: {:?}", e))?;

        let result = deploy_account(deploy_args, None, None, config).await?;
        Ok((result, credential))
    }

    #[tokio::test]
    #[ignore = "This test doesn't work need to investigate"]
    async fn test_register_apple_passkey() -> Result<()> {
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;

        let mut auth_stack = create_auth_stack();
        let rp_id = "example.com";
        let user_name = "user123";
        let user_display_name = "User 123";
        let challenge: Vec<u8> = Sha256::digest(b"random challenge").to_vec();

        let (deploy_result, credential) = deploy_account_with_apple_passkey(
            &config,
            &mut auth_stack,
            rp_id,
            user_name,
            user_display_name,
            challenge,
        )
        .await?;

        println!("Deployed account address: {:?}", deploy_result.address);
        println!(
            "Transaction receipt: {:?}",
            deploy_result.transaction_receipt_json
        );
        println!("Registered credential ID: {:?}", credential.id);

        let all_credentials = auth_stack
            .client
            .authenticator()
            .store()
            .find_credentials(None, rp_id)
            .await
            .map_err(|e| eyre::eyre!("Error finding credentials: {:?}", e))?;
        println!(
            "XDB - test - All credentials after registration: {all_credentials:?}"
        );

        let passkey = find_credentials(
            &mut auth_stack,
            rp_id,
            credential.id.as_bytes().to_vec().into(),
        )
        .await
        .map_err(|e| eyre::eyre!("Error finding credentials: {:?}", e))?;
        println!("Found passkey: {passkey:?}");

        drop(anvil_zksync);

        Ok(())
    }
}
