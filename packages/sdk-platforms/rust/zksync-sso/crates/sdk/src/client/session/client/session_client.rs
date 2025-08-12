use crate::{
    client::session::client::session_client::signature::{
        add_empty_signature_if_not_provided::add_empty_signature_if_not_provided,
        sign_eip_712_session_transaction::sign_eip_712_session_transaction,
    },
    config::Config,
    utils::{
        alloy::extensions::ProviderExt,
        manual_build_transaction::transaction_builder::populate_tx_request,
        session::session_lib::session_spec::SessionSpec,
    },
};
use alloy::{
    network::TransactionBuilder,
    primitives::{Address, FixedBytes},
    providers::Provider,
};
use alloy_zksync::{
    network::{
        receipt_response::ReceiptResponse,
        transaction_request::TransactionRequest,
    },
    provider::zksync_provider,
};
use log::debug;

pub mod signature;
pub mod timestamp;

pub struct SessionClient {
    pub account_address: Address,
    pub session_key: FixedBytes<32>,
    pub session_config: SessionSpec,
    pub config: Config,
}

impl SessionClient {
    pub fn new(
        account_address: Address,
        session_key: FixedBytes<32>,
        session_config: SessionSpec,
        config: Config,
    ) -> eyre::Result<Self> {
        Ok(Self { account_address, session_key, session_config, config })
    }

    pub async fn send_transaction(
        &self,
        mut tx_request: TransactionRequest,
    ) -> eyre::Result<ReceiptResponse> {
        debug!("SessionClient.send_transaction");

        debug!("  tx_request: {tx_request:?}");

        let account_address = self.account_address;
        debug!("  account_address: {account_address:?}");

        tx_request = tx_request.with_from(account_address);

        let from =
            tx_request.from().ok_or(eyre::eyre!("From address not found"))?;

        let to = tx_request.to().ok_or(eyre::eyre!("To address not found"))?;
        let value = tx_request.value().ok_or(eyre::eyre!("Value not found"))?;
        debug!("  From: {from}");
        debug!("  To: {to:?}");
        debug!("  Value: {value:?}");

        // Set empty custom signature for estimation if not provided
        tx_request = add_empty_signature_if_not_provided(
            tx_request,
            to,
            self.config.clone(),
            self.session_config.clone(),
        )?;

        // Populate the transaction with necessary fields (gas, nonce, etc.)
        let populated_tx =
            populate_tx_request(tx_request, &self.config).await?;

        // Sign the transaction
        let signed_raw_tx = sign_eip_712_session_transaction(
            populated_tx,
            self.account_address,
            self.session_key,
            self.session_config.clone(),
            self.config.clone(),
        )
        .await?;

        debug!(
            "SessionClient.send_transaction - signed_raw_tx: {signed_raw_tx:?}"
        );

        // Create provider to send the raw transaction
        let provider = {
            let node_url = self.config.node_url.clone();
            zksync_provider().with_recommended_fillers().on_http(node_url)
        };

        // Send the raw signed transaction
        debug!("Sending raw transaction. raw tx: {signed_raw_tx:?}");
        let pending_tx = provider.send_raw_transaction(&signed_raw_tx).await?;

        let tx_hash = pending_tx.tx_hash().to_owned();

        debug!("SessionClient.send_transaction - tx hash: {tx_hash}");

        // Wait for the transaction receipt
        let receipt = provider.wait_for_transaction_receipt(tx_hash).await?;

        debug!("SessionClient.send_transaction - receipt: {receipt:?}");

        Ok(receipt)
    }
}
