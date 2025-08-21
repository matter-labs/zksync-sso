use crate::{
    account::transaction::Transaction,
    config,
    native_apis::{PasskeyAuthenticator, PasskeyAuthenticatorAsync},
};
use futures::future::BoxFuture;
use log::debug;
use sdk::api::account::send::send_transaction as sdk_send_transaction;
use std::sync::Arc;

pub mod prepare;

#[derive(Debug, uniffi::Record)]
pub struct SendTransactionResult {
    pub tx_hash: String,
    pub receipt_json: String,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum SendTransactionError {
    #[error("{0}")]
    SendTransaction(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
}

impl From<sdk::api::account::send::SendTransactionResult>
    for SendTransactionResult
{
    fn from(result: sdk::api::account::send::SendTransactionResult) -> Self {
        Self { tx_hash: result.tx_hash, receipt_json: result.receipt_json }
    }
}

type SignFn = Box<
    dyn Fn(&[u8]) -> BoxFuture<'static, Result<Vec<u8>, String>>
        + Send
        + Sync
        + 'static,
>;

#[uniffi::export(async_runtime = "tokio")]
pub async fn send_transaction(
    transaction: Transaction,
    authenticator: Arc<dyn PasskeyAuthenticator + 'static>,
    config: config::Config,
) -> Result<SendTransactionResult, SendTransactionError> {
    debug!("XDB send_transaction - transaction: {transaction:?}");
    let tx: sdk::api::account::transaction::Transaction =
        transaction.try_into().map_err(
            |e: crate::account::transaction::TransactionConversionError| {
                SendTransactionError::SendTransaction(e.to_string())
            },
        )?;

    debug!("XDB send_transaction - tx: {tx:?}");

    let authenticator = authenticator.clone();
    let sign_message: SignFn = Box::new(
        move |message: &[u8]| -> BoxFuture<'static, Result<Vec<u8>, String>> {
            let message_owned = message.to_vec();
            let auth = authenticator.clone();
            debug!(
                "XDB send_transaction - sign_message - message_owned: {message_owned:?}"
            );
            Box::pin(async move {
                debug!(
                    "XDB send_transaction - sign_message - sign_message closure"
                );
                auth.sign_message(message_owned).map_err(|e| e.to_string())
            })
        },
    );

    let sdk_config = config.try_into().map_err(|e: config::ConfigError| {
        SendTransactionError::SendTransaction(e.to_string())
    })?;

    sdk_send_transaction(tx, sign_message, &sdk_config)
        .await
        .map_err(|e| SendTransactionError::SendTransaction(e.to_string()))
        .map(Into::into)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn send_transaction_async_signer(
    transaction: Transaction,
    authenticator: Arc<dyn PasskeyAuthenticatorAsync + 'static>,
    config: config::Config,
) -> Result<SendTransactionResult, SendTransactionError> {
    debug!("XDB send_transaction_async_signer - transaction: {transaction:?}");
    let tx: sdk::api::account::transaction::Transaction =
        transaction.try_into().map_err(
            |e: crate::account::transaction::TransactionConversionError| {
                SendTransactionError::SendTransaction(e.to_string())
            },
        )?;

    debug!("XDB send_transaction_async_signer - tx: {tx:?}");

    let authenticator = authenticator.clone();
    let sign_message: SignFn = Box::new(
        move |message: &[u8]| -> BoxFuture<'static, Result<Vec<u8>, String>> {
            let message_owned = message.to_vec();
            let auth = authenticator.clone();
            Box::pin(async move {
                auth.sign_message(message_owned)
                    .await
                    .map_err(|e| e.to_string())
            })
        },
    );

    let sdk_config = config.try_into().map_err(|e: config::ConfigError| {
        SendTransactionError::SendTransaction(e.to_string())
    })?;

    sdk_send_transaction(tx, sign_message, &sdk_config)
        .await
        .map_err(|e| SendTransactionError::SendTransaction(e.to_string()))
        .map(Into::into)
}
