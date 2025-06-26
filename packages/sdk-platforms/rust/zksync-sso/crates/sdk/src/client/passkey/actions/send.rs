use crate::{
    config::Config,
    utils::{
        alloy::passkey_raw_signer::PasskeySigningRawBackend,
        manual_build_transaction::transaction_builder::{
            build_raw_tx, populate_tx_request,
        },
    },
};
use alloy::{network::TransactionBuilder, providers::Provider};
use alloy_zksync::{
    network::{
        receipt_response::ReceiptResponse as ZKReceiptResponse,
        transaction_request::TransactionRequest,
    },
    provider::zksync_provider,
};
use log::debug;

pub mod prepare;
pub mod sign;

pub async fn send_transaction<S>(
    transaction_request: TransactionRequest,
    signer: S,
    config: &Config,
) -> eyre::Result<ZKReceiptResponse>
where
    S: PasskeySigningRawBackend,
{
    debug!("XDB client::passkey::actions::send::send_transaction");
    debug!("    XDB transaction: {:?}", transaction_request);
    debug!("    XDB from: {:?}", transaction_request.from());
    debug!("    XDB to: {:?}", transaction_request.to());
    debug!("    XDB value: {:?}", transaction_request.value());

    debug!(
        "XDB client::passkey::actions::send::send_transaction - tx: {:?}",
        transaction_request
    );

    let filled_tx = populate_tx_request(transaction_request, config).await?;

    let signed_tx = signer.sign_transaction(&filled_tx, config.clone()).await?;

    let raw_tx = build_raw_tx(signed_tx)?;

    let provider = zksync_provider()
        .with_recommended_fillers()
        .on_http(config.node_url.clone());

    let pending_tx = provider.send_raw_transaction(&raw_tx).await?;

    debug!(
        "XDB client::passkey::actions::send::send_transaction - pending_tx: {:?}",
        pending_tx
    );

    let receipt = pending_tx.get_receipt().await?;
    debug!(
        "XDB client::passkey::actions::send::send_transaction - receipt: {receipt:#?}"
    );

    Ok(receipt)
}
