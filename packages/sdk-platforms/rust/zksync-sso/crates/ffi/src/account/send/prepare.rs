use crate::config;
use sdk::api::utils::parse_address;

#[derive(Debug, uniffi::Record)]
pub struct PreparedTransaction {
    pub transaction_request_json: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub display_fee: String,
}

pub struct PreparedTransactionWrapper(
    sdk::api::account::send::prepare::PreparedTransaction,
);

impl TryFrom<PreparedTransactionWrapper> for PreparedTransaction {
    type Error = PrepareTransactionError;

    fn try_from(
        prepared_tx: PreparedTransactionWrapper,
    ) -> Result<Self, Self::Error> {
        let prepared_tx = prepared_tx.0;
        Ok(PreparedTransaction {
            transaction_request_json: serde_json::to_string(
                &prepared_tx.transaction_request,
            )
            .map_err(|e| {
                PrepareTransactionError::PrepareTransaction(e.to_string())
            })?,
            from: prepared_tx.from,
            to: prepared_tx.to,
            value: prepared_tx.value,
            display_fee: prepared_tx.display_fee,
        })
    }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum PrepareTransactionError {
    #[error("{0}")]
    PrepareTransaction(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn prepare_send_transaction(
    transaction: super::Transaction,
    from: String,
    config: config::Config,
) -> Result<PreparedTransaction, PrepareTransactionError> {
    let transaction: sdk::api::account::transaction::Transaction =
        transaction.try_into().map_err(|e| match e {
            super::SendTransactionError::InvalidAddress(e) => {
                PrepareTransactionError::InvalidAddress(e)
            }
            super::SendTransactionError::SendTransaction(e) => {
                PrepareTransactionError::PrepareTransaction(e)
            }
        })?;

    let from = parse_address(&from)
        .map_err(|e| PrepareTransactionError::InvalidAddress(e.to_string()))?;

    sdk::api::account::send::prepare::prepare_send_transaction(
        transaction,
        from,
        &(config.try_into()
            as Result<sdk::config::Config, config::ConfigError>)
            .map_err(|e: config::ConfigError| {
                PrepareTransactionError::PrepareTransaction(e.to_string())
            })?,
    )
    .await
    .map_err(|e| PrepareTransactionError::PrepareTransaction(e.to_string()))
    .and_then(|prepared_tx| {
        PreparedTransaction::try_from(PreparedTransactionWrapper(prepared_tx))
    })
}
