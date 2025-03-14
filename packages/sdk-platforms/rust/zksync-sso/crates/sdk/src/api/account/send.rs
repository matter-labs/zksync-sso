use crate::{
    client::passkey::actions::send::sign::SignerWithMessage, config::Config,
};
use alloy::network::ReceiptResponse;
use alloy::{
    network::TransactionBuilder,
    primitives::{Address, U256},
};
use alloy_zksync::network::transaction_request::TransactionRequest;
use std::{fmt::Debug, future::Future};

pub mod prepare;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Transaction {
    pub to: Address,
    pub value: String,
    pub from: Address,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct SendTransactionResult {
    pub tx_hash: String,
    pub receipt_json: String,
}

pub async fn send_transaction<F, Fut>(
    transaction: Transaction,
    sign_message: F,
    config: &Config,
) -> eyre::Result<SendTransactionResult>
where
    F: Fn(&[u8]) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Vec<u8>, ()>> + Send,
{
    let backend: SignerWithMessage<F> = SignerWithMessage::new(sign_message);
    println!(
        "XDB api::account::send::send_transaction - transaction: {:?}",
        transaction
    );

    let to = transaction.to;
    let value = transaction.value.parse::<U256>()?;
    let from = transaction.from;

    let transaction_request = TransactionRequest::default()
        .with_from(from)
        .with_to(to)
        .with_value(value);

    let receipt = crate::client::passkey::actions::send::send_transaction(
        transaction_request,
        backend,
        config,
    )
    .await?;

    let tx_hash = receipt.transaction_hash().to_string();
    let receipt_json = serde_json::to_string(&receipt)?;

    let result = SendTransactionResult { tx_hash, receipt_json };

    println!("XDB api::account::send::send_transaction - result: {:?}", result);

    Ok(result)
}

pub async fn send_transaction_fnonce_signer<F>(
    transaction: Transaction,
    sign_message: F,
    config: &Config,
) -> eyre::Result<SendTransactionResult>
where
    F: FnOnce(&[u8]) -> Result<Vec<u8>, ()> + Send + Sync + 'static,
{
    use crate::client::passkey::actions::send::sign::SignerWithMessageOnce;
    let backend = SignerWithMessageOnce::new(sign_message);
    println!(
        "XDB api::account::send::send_transaction - transaction: {:?}",
        transaction
    );

    let to = transaction.to;
    let value = transaction.value.parse::<U256>()?;
    let from = transaction.from;

    let transaction_request = TransactionRequest::default()
        .with_from(from)
        .with_to(to)
        .with_value(value);

    let receipt = crate::client::passkey::actions::send::send_transaction(
        transaction_request,
        backend,
        config,
    )
    .await?;

    let tx_hash = receipt.transaction_hash().to_string();
    let receipt_json = serde_json::to_string(&receipt)?;

    let result = SendTransactionResult { tx_hash, receipt_json };

    println!("XDB api::account::send::send_transaction - result: {:?}", result);

    Ok(result)
}
