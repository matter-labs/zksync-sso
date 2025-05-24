use crate::config::Config;
use alloy::{
    consensus::SignableTransaction,
    network::TxSigner,
    primitives::{Address, PrimitiveSignature as Signature},
};
use alloy_zksync::network::transaction_request::TransactionRequest;
use async_trait::async_trait;
use futures::future::BoxFuture;
use std::fmt::Debug;

#[async_trait]
pub trait PasskeySigningRawBackend: Send + Sync + Debug {
    async fn sign_transaction(
        &self,
        tx: &TransactionRequest,
        config: Config,
    ) -> eyre::Result<TransactionRequest>;
}
