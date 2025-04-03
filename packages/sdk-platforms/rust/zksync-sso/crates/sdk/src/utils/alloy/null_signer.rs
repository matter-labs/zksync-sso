use alloy::consensus::SignableTransaction;
use alloy::network::TxSigner;
use alloy::primitives::{
    Address, ChainId, PrimitiveSignature as Signature, B256,
};
use alloy::signers::{
    Error as SignerError, Result as SignerResult, SignerSync,
};
use async_trait::async_trait;
use std::fmt;

/// A signer that doesn't actually sign anything, but returns a dummy signature.
#[derive(Clone)]
pub struct NullSigner {
    address: Address,
    chain_id: Option<ChainId>,
}

impl NullSigner {
    pub fn new(address: Address) -> Self {
        Self { address, chain_id: None }
    }

    pub fn with_chain_id(address: Address, chain_id: ChainId) -> Self {
        Self { address, chain_id: Some(chain_id) }
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn set_chain_id(&mut self, chain_id: ChainId) {
        self.chain_id = Some(chain_id);
    }
}

impl fmt::Debug for NullSigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NullSigner")
            .field("address", &self.address)
            .field("chain_id", &self.chain_id)
            .finish()
    }
}

impl SignerSync for NullSigner {
    fn sign_hash_sync(&self, _hash: &B256) -> SignerResult<Signature> {
        // Create a dummy signature (all zeros)
        let dummy_signature_bytes = [0u8; 65];
        Signature::try_from(&dummy_signature_bytes[..]).map_err(|e| {
            SignerError::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create dummy signature: {}", e),
            )))
        })
    }

    fn chain_id_sync(&self) -> Option<ChainId> {
        self.chain_id
    }
}

#[async_trait]
impl TxSigner<Signature> for NullSigner {
    fn address(&self) -> Address {
        self.address
    }

    async fn sign_transaction(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> SignerResult<Signature> {
        let hash = tx.signature_hash();

        self.sign_hash_sync(&hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::U256;

    #[test]
    fn test_null_signer() {
        let address = Address::ZERO;
        let signer = NullSigner::new(address);

        assert_eq!(signer.address(), address);

        // Test signing
        let hash = B256::ZERO;
        let signature = signer.sign_hash_sync(&hash).unwrap();

        // Verify the signature is all zeros
        assert_eq!(signature.r(), U256::ZERO);
        assert_eq!(signature.s(), U256::ZERO);
        assert!(!signature.v());
    }

    #[test]
    fn test_null_signer_with_chain_id() {
        let address = Address::ZERO;
        let chain_id = ChainId::from(1u64);
        let signer = NullSigner::with_chain_id(address, chain_id);

        assert_eq!(signer.address(), address);
        assert_eq!(signer.chain_id_sync(), Some(chain_id));
    }

    #[test]
    fn test_set_chain_id() {
        let address = Address::ZERO;
        let mut signer = NullSigner::new(address);

        // Initially no chain ID
        assert_eq!(signer.chain_id_sync(), None);

        // Set chain ID
        let chain_id = ChainId::from(1u64);
        signer.set_chain_id(chain_id);

        // Verify chain ID was set
        assert_eq!(signer.chain_id_sync(), Some(chain_id));
    }
}
