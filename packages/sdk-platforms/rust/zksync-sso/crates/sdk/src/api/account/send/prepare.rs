use crate::{
    client::passkey::actions::send::prepare::prepare_transaction,
    config::Config,
};
use alloy_zksync::network::transaction_request::TransactionRequest;
use log::debug;

#[derive(Debug)]
pub struct PreparedTransaction {
    pub transaction_request: TransactionRequest,
    pub from: String,
    pub to: String,
    pub value: String,
    pub display_fee: String,
}

impl From<crate::client::passkey::actions::send::prepare::PreparedTransaction>
    for PreparedTransaction
{
    fn from(
        prepared_tx: crate::client::passkey::actions::send::prepare::PreparedTransaction,
    ) -> Self {
        Self {
            transaction_request: prepared_tx.transaction_request,
            from: prepared_tx.from,
            to: prepared_tx.to,
            value: prepared_tx.value,
            display_fee: prepared_tx.display_fee,
        }
    }
}

pub async fn prepare_send_transaction(
    transaction: super::Transaction,
    config: &Config,
) -> eyre::Result<PreparedTransaction> {
    debug!("XDB prepare_send_transaction - transaction: {transaction:?}");
    let transaction_request = transaction.try_into()?;
    debug!(
        "XDB prepare_send_transaction - transaction_request: {transaction_request:?}"
    );
    prepare_transaction(transaction_request, config).await.map(Into::into)
}
