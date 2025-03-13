use crate::{
    client::passkey::actions::send::prepare::prepare_transaction,
    config::Config,
};
use alloy::{
    network::TransactionBuilder,
    primitives::{Address, U256},
};
use alloy_zksync::network::transaction_request::TransactionRequest;

pub async fn prepare_send_transaction(
    transaction: super::Transaction,
    from: Address,
    config: &Config,
) -> eyre::Result<
    crate::client::passkey::actions::send::prepare::PreparedTransaction,
> {
    println!("XDB prepare_send_transaction - transaction: {:?}", transaction);
    let value = transaction.value.parse::<U256>()?;
    let transaction_request = TransactionRequest::default()
        .with_from(from)
        .with_to(transaction.to)
        .with_value(value);
    prepare_transaction(transaction_request, from, config).await
}
