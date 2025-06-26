use crate::config::Config;
use alloy_zksync::network::transaction_request::TransactionRequest;
use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait]
pub trait PasskeySigningRawBackend: Send + Sync + Debug {
    async fn sign_transaction(
        &self,
        tx: &TransactionRequest,
        config: Config,
    ) -> eyre::Result<TransactionRequest>;
}
