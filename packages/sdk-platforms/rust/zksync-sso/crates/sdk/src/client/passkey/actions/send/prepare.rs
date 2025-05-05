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
    println!("XDB prepare_transaction");
    println!("    XDB transaction_request: {:?}", transaction_request);
    println!("    XDB config: {:?}", config);

    let filled_tx = populate_tx_request(transaction_request, config).await?;
    println!("XDB prepare_transaction - filled_tx: {:?}", filled_tx);

    let to = filled_tx.to().ok_or(eyre::eyre!("Value is required"))?;
    println!("XDB prepare_transaction - to: {:?}", to);

    let value = filled_tx.value().ok_or(eyre::eyre!("Value is required"))?;
    println!("XDB prepare_transaction - value: {:?}", value);

    let from = filled_tx.from().ok_or(eyre::eyre!("From is required"))?;
    println!("XDB prepare_transaction - from: {:?}", from);

    let prepared_transaction = PreparedTransaction {
        from: format!("{:?}", from),
        to: format!("{:?}", to),
        value: value.to_string(),
        display_fee: fee::calculate_display_fee(&filled_tx),
        transaction_request: filled_tx,
    };

    println!(
        "XDB prepare_transaction - prepared_transaction: {:?}",
        prepared_transaction
    );

    Ok(prepared_transaction)
}
