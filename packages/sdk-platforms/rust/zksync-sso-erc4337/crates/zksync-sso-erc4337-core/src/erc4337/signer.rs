use alloy::primitives::{Bytes, FixedBytes};
use eyre;
use std::sync::Arc;

pub type SignatureProvider =
    Arc<dyn Fn(FixedBytes<32>) -> eyre::Result<Bytes> + Send + Sync>;

#[derive(Clone)]
pub struct Signer {
    pub stub_signature: Bytes,
    pub provider: SignatureProvider,
}
