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

#[derive(Debug)]
pub struct PasskeyRawSigner<B: std::fmt::Debug> {
    address: Address,
    backend: B,
}

impl<B> PasskeyRawSigner<B>
where
    B: PasskeySigningRawBackend + std::fmt::Debug,
{
    pub fn new(address: Address, backend: B) -> Self {
        println!(
            "XDB - alloy::utils::passkey_signer::PasskeySigner::new - address: {:?}",
            address
        );
        println!(
            "XDB - alloy::utils::passkey_signer::PasskeySigner::new - backend: {:?}",
            backend
        );
        Self { address, backend }
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub async fn sign_transaction(
        &self,
        tx: &TransactionRequest,
        config: Config,
    ) -> alloy::signers::Result<TransactionRequest> {
        println!(
            "XDB - alloy::utils::passkey_signer::PasskeySigner::sign_transaction - address: {:?}",
            self.address
        );
        println!(
            "XDB - alloy::utils::passkey_signer::PasskeySigner::sign_transaction - backend: {:?}",
            self.backend
        );
        self.backend.sign_transaction(tx, config).await.map_err(|e| {
            alloy::signers::Error::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))
        })
    }
}

pub type SigningFn = Box<
    dyn Fn(
            &mut dyn SignableTransaction<Signature>,
        ) -> BoxFuture<'static, eyre::Result<Signature>>
        + Send
        + Sync,
>;

pub struct PasskeySignerWithClosure {
    address: Address,
    sign_fn: SigningFn,
}

impl PasskeySignerWithClosure {
    pub fn new(address: Address, sign_fn: SigningFn) -> Self {
        Self { address, sign_fn }
    }
}

impl std::fmt::Debug for PasskeySignerWithClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasskeySignerWithClosure")
            .field("address", &self.address)
            .field("sign_fn", &"<closure>")
            .finish()
    }
}

#[async_trait]
impl TxSigner<Signature> for PasskeySignerWithClosure {
    fn address(&self) -> Address {
        self.address
    }

    async fn sign_transaction(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> alloy::signers::Result<Signature> {
        (self.sign_fn)(tx).await.map_err(|e| {
            alloy::signers::Error::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))
        })
    }
}
