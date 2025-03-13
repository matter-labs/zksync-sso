use crate::{
    config::Config, utils::alloy::passkey_raw_signer::PasskeySigningRawBackend,
};
use alloy_zksync::network::transaction_request::TransactionRequest;
use async_trait::async_trait;
use std::{fmt::Debug, future::Future};
use tokio::sync::Mutex;

pub mod hash_signature_response;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct SendTransactionResult {
    pub tx_hash: String,
    pub receipt: String,
}

pub struct SignerWithMessage<F> {
    sign_message: F,
}

pub struct SignerWithMessageOnce<F> {
    sign_message: Mutex<Option<F>>,
}

impl<F> SignerWithMessage<F> {
    pub fn new(sign_message: F) -> Self {
        Self { sign_message }
    }
}

impl<F> SignerWithMessageOnce<F> {
    pub fn new(sign_message: F) -> Self {
        Self { sign_message: Mutex::new(Some(sign_message)) }
    }
}

impl<F> Debug for SignerWithMessage<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignerWithMessage")
            .field("sign_message", &"<closure>")
            .finish()
    }
}

impl<F> Debug for SignerWithMessageOnce<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignerWithMessageOnce")
            .field("sign_message", &"<closure>")
            .finish()
    }
}

#[async_trait]
impl<F, Fut> PasskeySigningRawBackend for SignerWithMessage<F>
where
    F: Fn(&[u8]) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Vec<u8>, ()>> + Send,
{
    async fn sign_transaction(
        &self,
        tx: &TransactionRequest,
        config: Config,
    ) -> eyre::Result<TransactionRequest> {
        let mut tx = tx.clone();

        let digest_hash =
            crate::client::passkey::account::transaction_digest::get_digest(
                tx.to_owned(),
            )
            .map_err(|e| {
                eyre::eyre!("Error getting transaction digest: {:?}", e)
            })?;
        let hash = digest_hash.0.to_vec();

        println!("XDB - sign_transaction hash: {:?}", hash);

        let signature_response = (self.sign_message)(hash.as_slice())
            .await
            .map_err(|_| eyre::eyre!("Signing failed"))?;

        println!(
            "XDB - sign_transaction signature_response: {:?}",
            signature_response
        );

        let signature_encoded =
            hash_signature_response::hash_signature_response_format(
                signature_response,
                &config,
            )?;

        tx.set_custom_signature(signature_encoded.into());

        Ok(tx.to_owned())
    }
}

#[async_trait]
impl<F> PasskeySigningRawBackend for SignerWithMessageOnce<F>
where
    F: FnOnce(&[u8]) -> Result<Vec<u8>, ()> + Send + Sync,
{
    async fn sign_transaction(
        &self,
        tx: &TransactionRequest,
        config: Config,
    ) -> eyre::Result<TransactionRequest> {
        let mut tx = tx.clone();

        let digest_hash =
            crate::client::passkey::account::transaction_digest::get_digest(
                tx.to_owned(),
            )
            .map_err(|e| {
                eyre::eyre!("Error getting transaction digest: {:?}", e)
            })?;
        let hash = digest_hash.0.to_vec();

        println!("XDB - sign_transaction hash: {:?}", hash);

        let signature_response =
            (self.sign_message.lock().await.take().unwrap())(hash.as_slice())
                .map_err(|_| eyre::eyre!("Signing failed"))?;

        println!(
            "XDB - sign_transaction signature_response: {:?}",
            signature_response
        );

        let signature_encoded =
            hash_signature_response::hash_signature_response_format(
                signature_response,
                &config,
            )?;

        tx.set_custom_signature(signature_encoded.into());

        Ok(tx.to_owned())
    }
}
