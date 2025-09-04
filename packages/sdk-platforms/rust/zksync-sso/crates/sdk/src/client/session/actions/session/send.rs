use crate::{
    config::Config,
    utils::{
        manual_build_transaction::transaction_builder::{
            build_raw_tx, populate_tx_request,
        },
        transaction::{Transaction, transaction_digest::create_domain},
    },
};
use alloy::{
    network::TransactionBuilder,
    primitives::{Address, Bytes, FixedBytes, hex},
    providers::{PendingTransactionBuilder, Provider},
    signers::SignerSync,
    sol_types::SolStruct,
};
use alloy_zksync::{
    network::{Zksync, transaction_request::TransactionRequest},
    provider::zksync_provider,
};
use async_trait::async_trait;
use log::debug;

#[async_trait]
pub trait SessionSend: Send + Sync {
    async fn send(
        &self,
        transaction_request: TransactionRequest,
    ) -> eyre::Result<PendingTransactionBuilder<Zksync>>;
}

pub type SignFn =
    Box<dyn Fn(&FixedBytes<32>) -> eyre::Result<Vec<u8>> + Send + Sync>;

pub fn sign_fn_from_signer<S: SignerSync + Send + Sync + 'static>(
    signer: S,
) -> SignFn {
    Box::new(move |hash: &FixedBytes<32>| -> eyre::Result<Vec<u8>> {
        let signature = signer
            .sign_hash_sync(hash)
            .map_err(|e| eyre::eyre!("Failed to sign hash: {}", e))?;
        Ok(signature.as_bytes().to_vec())
    })
}

pub(crate) struct SessionSendImpl<'a> {
    pub config: &'a Config,
    pub sign_hash: SignFn,
    pub deployed_account_address: &'a Address,
}

impl<'a> SessionSendImpl<'a> {
    pub fn new(
        config: &'a Config,
        sign_fn: SignFn,
        deployed_account_address: &'a Address,
    ) -> Self {
        Self { config, sign_hash: sign_fn, deployed_account_address }
    }
}

#[async_trait]
impl<'a> SessionSend for SessionSendImpl<'a> {
    async fn send(
        &self,
        transaction_request: TransactionRequest,
    ) -> eyre::Result<
        alloy::providers::PendingTransactionBuilder<
            alloy_zksync::network::Zksync,
        >,
    > {
        let transaction_request = transaction_request
            .clone()
            .with_from(self.deployed_account_address.to_owned());

        let populated_tx =
            populate_tx_request(transaction_request, self.config).await?;

        let signed_tx =
            sign_session_management_tx(populated_tx.clone(), &self.sign_hash)?;

        let raw_tx = build_raw_tx(signed_tx)?;
        debug!("ecdsa_sign_tx - raw_tx: {:?}", raw_tx);
        debug!("         raw_tx hex: 0x{:?}", hex::encode(&raw_tx));

        let provider = zksync_provider()
            .with_recommended_fillers()
            .on_http(self.config.node_url.clone());

        let pending_tx = provider.send_raw_transaction(&raw_tx).await?;

        Ok(pending_tx)
    }
}

/// Sign a transaction request using a closure
fn sign_session_management_tx(
    tx_request: TransactionRequest,
    sign_fn: &SignFn,
) -> eyre::Result<TransactionRequest> {
    let mut tx_request = tx_request.clone();

    let signing_hash = hash_session_management_tx(tx_request.clone())?;
    debug!("sign_session_management_tx - signing_hash: {signing_hash:?}");
    debug!(
        "sign_session_management_tx - signing_hash hex: {:?}",
        hex::encode(signing_hash)
    );

    let signature = sign_fn(&signing_hash)?;
    debug!("sign_session_management_tx - signature: {signature:?}");

    let signature_bytes: Bytes = signature.into();
    debug!(
        "sign_session_management_tx - signature_bytes: {:?}",
        signature_bytes.to_vec()
    );

    tx_request.set_custom_signature(signature_bytes);

    debug!("sign_session_management_tx - signedtx_request: {tx_request:?}");

    Ok(tx_request)
}

fn hash_session_management_tx(
    tx_request: TransactionRequest,
) -> eyre::Result<FixedBytes<32>> {
    let chain_id =
        tx_request.chain_id().ok_or(eyre::eyre!("Chain ID is required"))?;
    debug!("hash_session_management_tx - chain_id: {chain_id:?}");

    let domain = create_domain(chain_id);
    debug!("hash_session_management_tx - domain: {domain:?}");

    let payload: Transaction = tx_request.clone().try_into()?;
    debug!("hash_session_management_tx - payload: {payload:?}");

    let signing_hash = payload.eip712_signing_hash(&domain);

    Ok(signing_hash)
}
