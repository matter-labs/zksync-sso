use crate::config::Config;
use alloy::{
    consensus::SignableTransaction,
    eips::Encodable2718,
    network::TransactionBuilder,
    primitives::{Bytes, PrimitiveSignature, U256},
    providers::{
        SendableTx,
        SendableTx::{Builder, Envelope},
    },
};
use alloy_zksync::{
    network::{
        Zksync,
        transaction_request::TransactionRequest,
        tx_envelope::TxEnvelope,
        unsigned_tx::eip712::{Eip712Meta, TxEip712},
    },
    provider::zksync_provider,
};
use log::debug;
use passkey::{
    authenticator::{
        Authenticator, CredentialStore, MemoryStore, MockUserValidationMethod,
        UserCheck,
    },
    client::{Client, DefaultClientData, WebauthnError},
    types::{
        Passkey,
        ctap2::*,
        webauthn::{
            self, AttestationConveyancePreference, CredentialRequestOptions,
            PublicKeyCredentialDescriptor, PublicKeyCredentialRequestOptions,
            PublicKeyCredentialType, UserVerificationRequirement,
        },
    },
};
use public_suffix::PublicSuffixList;
use std::{fmt::Debug, sync::Arc};
use tokio::sync::Mutex;
use url::Url;

#[derive(Clone, Debug)]
pub struct TransactionRequestWrapper(pub TransactionRequest);

impl From<TransactionRequestWrapper> for TxEip712 {
    fn from(tx: TransactionRequestWrapper) -> Self {
        let tx = tx.clone().0;
        let gas_per_pubdata = tx.gas_per_pubdata().unwrap_or_default();
        let factory_deps: Vec<Bytes> =
            tx.factory_deps().map(ToOwned::to_owned).unwrap_or_default();

        let custom_signature = tx.custom_signature();
        let paymaster_params = tx.paymaster_params();
        let eip712_meta = Eip712Meta {
            gas_per_pubdata,
            factory_deps,
            custom_signature: custom_signature.map(ToOwned::to_owned),
            paymaster_params: paymaster_params.map(ToOwned::to_owned),
        };
        let chain_id = tx.chain_id().unwrap_or_default();
        let from = tx.from().unwrap_or_default();
        let to = tx.to().unwrap_or_default();
        let nonce_u64 = tx.nonce().unwrap_or_default();
        let nonce = U256::from(nonce_u64);
        let value = tx.value().unwrap_or_default();
        let gas = tx.gas_limit().unwrap_or_default();
        let max_fee_per_gas = tx.max_fee_per_gas().unwrap_or_default();
        let max_priority_fee_per_gas =
            tx.max_priority_fee_per_gas().unwrap_or_default();
        let input = tx.input().map(ToOwned::to_owned).unwrap_or_default();

        TxEip712 {
            chain_id,
            from,
            to,
            nonce,
            value,
            gas,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            input,
            eip712_meta: Some(eip712_meta),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SendableTxWrapper(pub SendableTx<Zksync>);

impl From<SendableTxWrapper> for TransactionRequest {
    fn from(sendable_tx: SendableTxWrapper) -> Self {
        match sendable_tx.0 {
            Builder(tx) => tx,
            Envelope(tx) => tx.into(),
        }
    }
}

pub(crate) async fn populate_tx_request(
    tx_request: TransactionRequest,
    config: &Config,
) -> eyre::Result<TransactionRequest> {
    let provider = zksync_provider()
        .with_recommended_fillers()
        .on_http(config.node_url.clone());

    let mut tx_request: TransactionRequest = tx_request;
    tx_request.set_gas_per_pubdata(U256::from(50000));

    debug!(
        "XDB - populate_tx_request - going to fill transaction: {:?}",
        tx_request
    );
    let sendable_tx: alloy::providers::SendableTx<Zksync> =
        provider.fill(tx_request.clone()).await?;
    debug!(
        "XDB - populate_tx_request - transaction filled, sendable_tx: {:?}",
        sendable_tx
    );

    let mut tx: TransactionRequest = SendableTxWrapper(sendable_tx).into();

    debug!("XDB - populate_tx_request - transaction filled, tx: {:?}", tx);

    let max_priority_fee_per_gas = tx.max_fee_per_gas().unwrap_or_default();
    tx.set_max_priority_fee_per_gas(max_priority_fee_per_gas);

    tx.set_gas_limit(100000000);

    assert!(tx.gas_per_pubdata().unwrap() == U256::from(50000));

    debug!(
        "XDB - populate_tx_request - Built TransactionRequest tx: \n{:?}",
        tx
    );

    Ok(tx)
}

pub(crate) fn build_raw_tx(tx: TransactionRequest) -> eyre::Result<Vec<u8>> {
    let tx_eip712: TxEip712 = TransactionRequestWrapper(tx).into();
    let out = {
        let mut out = Vec::new();
        let dummy_signature_bytes = vec![0; 65];
        let dummy_signature: PrimitiveSignature =
            dummy_signature_bytes.as_slice().try_into()?;
        let signed_tx = tx_eip712.into_signed(dummy_signature);
        let envelope = TxEnvelope::Eip712(signed_tx);
        envelope.encode_2718(&mut out);
        out
    };
    debug!(
        "Encoded transaction with custom signature: 0x{}",
        hex::encode(&out)
    );
    Ok(out)
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct AuthStack {
    pub client: Arc<
        Mutex<Client<MemoryStore, MockUserValidationMethod, PublicSuffixList>>,
    >,
    pub rp_id: String,
    pub user_name: String,
    pub user_display_name: String,
    pub challenge: Vec<u8>,
}

#[allow(dead_code)]
pub fn create_auth_stack() -> AuthStack {
    let store = MemoryStore::new();
    let mut mock_uv = MockUserValidationMethod::new();
    mock_uv.expect_check_user().returning(|_, _, _| {
        Ok(UserCheck { presence: true, verification: true })
    });
    mock_uv.expect_is_verification_enabled().returning(|| Some(true));
    mock_uv.expect_is_presence_enabled().returning(|| true);

    let client =
        Client::new(Authenticator::new(Aaguid::new_empty(), store, mock_uv));

    AuthStack {
        client: Arc::new(Mutex::new(client)),
        rp_id: "example.com".to_string(),
        user_name: "user123".to_string(),
        user_display_name: "User 123".to_string(),
        challenge: vec![],
    }
}

#[allow(dead_code)]
async fn find_credentials(
    auth_stack: &mut AuthStack,
    rp_id: &str,
    credential_raw_id: Bytes,
) -> Result<Passkey, WebauthnError> {
    let ids = vec![passkey_types::webauthn::PublicKeyCredentialDescriptor {
        ty: PublicKeyCredentialType::PublicKey,
        id: credential_raw_id.to_vec().into(),
        transports: None,
    }];

    let credentials = auth_stack
        .client
        .lock()
        .await
        .authenticator()
        .store()
        .find_credentials(Some(&ids), rp_id)
        .await?;

    if credentials.is_empty() {
        return Err(WebauthnError::CredentialNotFound);
    }

    Ok(credentials.first().unwrap().to_owned())
}

#[allow(dead_code)]
fn get_origin(rp_id: &str) -> Url {
    Url::parse(&format!("https://{}", rp_id)).unwrap()
}

#[allow(dead_code)]
async fn authenticate_apple_passkey(
    auth_stack: &mut AuthStack,
    rp_id: &str,
    challenge: Vec<u8>,
    credential_raw_id: Vec<u8>,
) -> Result<
    webauthn::PublicKeyCredential<webauthn::AuthenticatorAssertionResponse>,
    WebauthnError,
> {
    let origin = get_origin(rp_id);

    let ids = vec![passkey_types::webauthn::PublicKeyCredentialDescriptor {
        ty: PublicKeyCredentialType::PublicKey,
        id: credential_raw_id.into(),
        transports: None,
    }];

    let passkeys = auth_stack
        .client
        .lock()
        .await
        .authenticator()
        .store()
        .find_credentials(Some(&ids), rp_id)
        .await?;
    debug!(
        "XDB - authenticate_apple_passkey - Available passkeys: {:?}",
        passkeys
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
        .lock()
        .await
        .authenticate(&origin, options, DefaultClientData)
        .await?;
    debug!(
        "XDB - authenticate_apple_passkey - Auth response credential ID: {:?}",
        auth_response.id
    );
    Ok(auth_response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::Config,
        utils::manual_build_transaction::register_passkey::tests::{
            AuthStack, authenticate_apple_passkey, create_auth_stack,
            deploy_account_with_apple_passkey, find_credentials,
        },
    };
    use alloy::primitives::address;
    use base64::Engine;
    use serde_json;
    use sha2::{Digest, Sha256};
    use std::io::Write;

    async fn sign(
        hash: &[u8],
        auth_stack: &mut AuthStack,
        rp_id: &str,
        credential_raw_id: Vec<u8>,
    ) -> Result<Bytes, String> {
        println!("XDB - sign - Starting sign function");
        std::io::stdout().flush().unwrap();

        let challenge = hash.to_vec();
        println!("XDB - sign - Challenge: {:?}", challenge);
        std::io::stdout().flush().unwrap();

        println!("XDB - sign - About to call authenticate_apple_passkey");
        std::io::stdout().flush().unwrap();

        let credential: passkey_types::webauthn::PublicKeyCredential<
            passkey_types::webauthn::AuthenticatorAssertionResponse,
        > = authenticate_apple_passkey(
            auth_stack,
            rp_id,
            challenge,
            credential_raw_id.to_vec(),
        )
        .await
        .map_err(|e| {
            println!("XDB - sign - Error authenticating passkey: {:?}", e);
            std::io::stdout().flush().unwrap();
            eyre::eyre!("Error authenticating passkey: {:?}", e)
        })
        .map_err(|e| e.to_string())?;

        println!("XDB - sign - Got credential: {:?}", credential);
        std::io::stdout().flush().unwrap();

        let response = credential.response;
        println!("XDB - sign - Got response: {:?}", response);
        std::io::stdout().flush().unwrap();

        let engine = base64::engine::general_purpose::STANDARD;

        let assertion = serde_json::json!({
            "attachment": "platform",
            "rawAuthenticatorData": engine.encode(response.authenticator_data.to_vec()),
            "userID": response.user_handle.map(|h| engine.encode(h.to_vec())).unwrap_or_else(|| engine.encode(vec![])),
            "signature": engine.encode(response.signature.to_vec()),
            "credentialID": engine.encode(credential.raw_id.to_vec()),
            "rawClientDataJSON": engine.encode(response.client_data_json.to_vec())
        });

        println!(
            "XDB - sign - assertion JSON: {}",
            serde_json::to_string_pretty(&assertion).unwrap()
        );
        std::io::stdout().flush().unwrap();

        let json_response = serde_json::to_string(&assertion).unwrap();
        let bytes: Bytes = json_response.into();

        println!("XDB - sign - Returning bytes: {:?}", bytes);
        std::io::stdout().flush().unwrap();

        Ok(bytes)
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "This test doesn't work need to investigate"]
    async fn test_build_tx() -> eyre::Result<()> {
        println!("Starting test_build_tx");
        std::io::stdout().flush().unwrap();

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            async {

                let config = Config::local();

                let mut auth_stack = create_auth_stack();
                let rp_id = "example.com";
                let user_name = "user123";
                let user_display_name = "User 123";
                let challenge = Sha256::digest(b"random challenge").to_vec();

                let (deploy_result, credential) = deploy_account_with_apple_passkey(
                    &config,
                    &mut auth_stack,
                    rp_id,
                    user_name,
                    user_display_name,
                    challenge,
                )
                .await?;
                let account_address = deploy_result.address;
                println!(
                    "XDB - test_build_tx - Account Address: {:?}",
                    account_address
                );
                std::io::stdout().flush().unwrap();

                let credential_raw_id = credential.raw_id;
                let credential_raw_id_vec = credential_raw_id.clone().to_vec();
                println!(
                    "XDB - test_build_tx - Credential ID: {:?}",
                    credential_raw_id_vec
                );
                std::io::stdout().flush().unwrap();

                let store = auth_stack.client.authenticator().store().clone();
                println!("XDB - test_build_tx - Store: {:?}", store);
                std::io::stdout().flush().unwrap();

                let passkey = store.get(&credential_raw_id.clone().to_vec());
                println!("XDB - test_build_tx - Passkey: {:?}", passkey);
                std::io::stdout().flush().unwrap();

                let passkey = find_credentials(
                    &mut auth_stack,
                    rp_id,
                    credential_raw_id,
                )
                .await
                .map_err(|e| eyre::eyre!("Error finding credentials: {:?}", e))?;
                println!("XDB - test_build_tx - Found passkey: {:?}", passkey);
                std::io::stdout().flush().unwrap();

                use crate::api::account::{
                    send::send_transaction_fnonce_signer,
                    transaction::Transaction,
                };
                use alloy::primitives::address;

                let from = address!("1234567890123456789012345678901234567890");
                let to = Some(address!("0987654321098765432109876543210987654321"));
                let value = U256::from(1000u64);

                let value_str = value.to_string();

                let transaction = Transaction { to, value: Some(value_str), from, input: None };
                let auth_stack_arc = Arc::new(Mutex::new(auth_stack));
                let rp_id = rp_id.to_string();
                let credential_raw_id = credential_raw_id_vec.clone();

                let rt = tokio::runtime::Handle::current();
                let sign_message = move |hash: &[u8]| {
                    let auth_stack_arc = Arc::clone(&auth_stack_arc);
                    let rp_id = rp_id.clone();
                    let credential_raw_id = credential_raw_id.clone();

                    println!("XDB - test_build_tx - Signing message going to block");
                    std::io::stdout().flush().unwrap();

                    tokio::task::block_in_place(|| {
                        println!("XDB - test_build_tx - Inside block_in_place");
                        std::io::stdout().flush().unwrap();

                        rt.block_on(async move {
                            println!("XDB - test_build_tx - Signing message going to acquire lock");
                            std::io::stdout().flush().unwrap();

                            let mut auth_stack = auth_stack_arc.lock().await;

                            println!("XDB - test_build_tx - Signing message going to sign");
                            std::io::stdout().flush().unwrap();

                            sign(hash, &mut auth_stack, &rp_id, credential_raw_id)
                                .await
                                .map(|bytes| bytes.to_vec())
                                .map_err(|e| e.to_string())
                        })
                    })
                };

                let _result =
                    send_transaction_fnonce_signer(transaction, sign_message, &config).await?;

                // drop(anvil_zksync);

                Ok(())
            }
        ).await;

        match result {
            Ok(inner_result) => inner_result,
            Err(_) => {
                println!("Test timed out after 30 seconds!");
                std::io::stdout().flush().unwrap();
                Err(eyre::eyre!("Test timed out"))
            }
        }
    }
}
