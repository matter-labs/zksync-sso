use crate::{account::transaction::Transaction, config};
use sdk::api::{
    account::{
        session::{client::SessionClient, session_lib::session_spec_from_json},
        transaction::Transaction as CoreTransaction,
    },
    utils::{decode_fixed_bytes_hex, parse_address},
};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum SessionClientError {
    #[error("{0}")]
    SessionClient(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Invalid session config: {0}")]
    InvalidSessionConfig(String),
    #[error("Invalid session key: {0}")]
    InvalidSessionKey(String),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    #[error("Invalid receipt: {0}")]
    InvalidReceipt(String),
    #[error("Invalid transaction request: {0}")]
    SendFailed(String),
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn send_session_transaction(
    account_address: String,
    session_key_hex: String,
    session_config_json: String,
    config: config::Config,
    transaction: crate::account::transaction::Transaction,
) -> Result<String, SessionClientError> {
    let session_client = SessionClientWrapper::new(
        account_address,
        session_key_hex,
        session_config_json,
        config,
    )
    .map_err(|e| SessionClientError::SessionClient(e.to_string()))?;
    session_client.send_transaction(transaction).await
}

struct SessionClientWrapper(SessionClient);

impl SessionClientWrapper {
    fn new(
        account_address: String,
        session_key_hex: String,
        session_config_json: String,
        config: config::Config,
    ) -> Result<Self, eyre::Report> {
        let account_address = parse_address(&account_address)
            .map_err(|e| SessionClientError::InvalidAddress(e.to_string()))?;
        let session_key = decode_fixed_bytes_hex::<32>(&session_key_hex)
            .map_err(|e| {
                SessionClientError::InvalidSessionKey(e.to_string())
            })?;
        let session_config = session_spec_from_json(&session_config_json)
            .map_err(|e| {
                SessionClientError::InvalidSessionConfig(e.to_string())
            })?;
        let session_client = SessionClient::new(
            account_address,
            session_key,
            session_config,
            config.try_into()?,
        )?;
        let session_client_wrapper = SessionClientWrapper(session_client);
        Ok(session_client_wrapper)
    }

    async fn send_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<String, SessionClientError> {
        let core_transaction: CoreTransaction =
            transaction.try_into().map_err(
                |e: crate::account::transaction::TransactionConversionError| {
                    SessionClientError::InvalidTransaction(e.to_string())
                },
            )?;
        let tx_request =
            core_transaction.try_into().map_err(|e: eyre::Report| {
                SessionClientError::InvalidTransaction(e.to_string())
            })?;
        let receipt = self
            .0
            .send_transaction(tx_request)
            .await
            .map_err(|e| SessionClientError::SendFailed(e.to_string()))?;
        let receipt_json = serde_json::to_string(&receipt)
            .map_err(|e| SessionClientError::InvalidReceipt(e.to_string()))?;
        Ok(receipt_json)
    }
}

impl From<SessionClient> for SessionClientWrapper {
    fn from(client: SessionClient) -> Self {
        SessionClientWrapper(client)
    }
}
