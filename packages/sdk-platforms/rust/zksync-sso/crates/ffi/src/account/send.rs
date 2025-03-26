use crate::config;
use crate::native_apis::PasskeyAuthenticator;
use futures::future::BoxFuture;
use sdk::utils::alloy::parse_address;
use std::sync::Arc;

pub mod prepare;

#[derive(Debug, uniffi::Record)]
pub struct Transaction {
    pub to: String,
    pub value: String,
    pub from: String,
}

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

impl TryFrom<Transaction> for sdk::api::account::send::Transaction {
    type Error = SendTransactionError;

    fn try_from(tx: Transaction) -> Result<Self, Self::Error> {
        Ok(Self {
            to: parse_address(&tx.to).map_err(|e| {
                SendTransactionError::InvalidAddress(e.to_string())
            })?,
            value: tx.value,
            from: parse_address(&tx.from).map_err(|e| {
                SendTransactionError::InvalidAddress(e.to_string())
            })?,
        })
    }
}

impl From<sdk::api::account::send::SendTransactionResult>
    for SendTransactionResult
{
    fn from(result: sdk::api::account::send::SendTransactionResult) -> Self {
        Self { tx_hash: result.tx_hash, receipt_json: result.receipt_json }
    }
}

type SignFn = Box<
    dyn Fn(&[u8]) -> BoxFuture<'static, Result<Vec<u8>, ()>>
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
    let tx: sdk::api::account::send::Transaction = transaction.try_into()?;

    let authenticator = authenticator.clone();
    let sign_message: SignFn = Box::new(
        move |message: &[u8]| -> BoxFuture<'static, Result<Vec<u8>, ()>> {
            let message_owned = message.to_vec();
            let auth = authenticator.clone();
            Box::pin(
                async move { auth.sign_message(message_owned).map_err(|_| ()) },
            )
        },
    );

    sdk::api::account::send::send_transaction(
        tx,
        sign_message,
        &(config.try_into()
            as Result<sdk::config::Config, config::ConfigError>)
            .map_err(|e: config::ConfigError| {
                SendTransactionError::SendTransaction(e.to_string())
            })?,
    )
    .await
    .map_err(|e| SendTransactionError::SendTransaction(e.to_string()))
    .map(Into::into)
}
