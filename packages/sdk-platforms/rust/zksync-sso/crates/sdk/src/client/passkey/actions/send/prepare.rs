use crate::utils::manual_build_transaction::transaction_builder::populate_tx_request;
use alloy::network::TransactionBuilder;
use alloy_zksync::network::transaction_request::TransactionRequest;
use std::fmt::Debug;

pub mod fee;

#[derive(Debug)]
pub struct PreparedTransaction {
    pub transaction_request: TransactionRequest,
    pub from: String,
    pub to: String,
    pub value: String,
    pub display_fee: String,
}

pub async fn prepare_transaction(
    transaction_request: TransactionRequest,
    config: &crate::config::Config,
) -> eyre::Result<PreparedTransaction> {
    println!("XDB client::passkey::actions::send::send_transaction");
    println!("    XDB transaction: {:?}", transaction_request);
    println!("    XDB config: {:?}", config);

    let filled_tx = populate_tx_request(transaction_request, config).await?;

    let to = filled_tx.to().ok_or(eyre::eyre!("Value is required"))?;
    let value = filled_tx.value().ok_or(eyre::eyre!("Value is required"))?;
    let from = filled_tx.from().ok_or(eyre::eyre!("From is required"))?;

    let prepared_transaction = PreparedTransaction {
        from: format!("{:?}", from),
        to: format!("{:?}", to),
        value: value.to_string(),
        display_fee: fee::calculate_display_fee(&filled_tx),
        transaction_request: filled_tx,
    };

    Ok(prepared_transaction)
}
