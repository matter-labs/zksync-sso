#[cfg(any(test, feature = "test-utilities"))]
pub mod test_utils;

use crate::erc4337::account::modular_smart_account::signers::eoa::{
    eoa_signature, stub_signature_eoa,
};
use alloy::primitives::{Address, Bytes, FixedBytes};
use eyre;
use std::{future::Future, pin::Pin, sync::Arc};

pub type SignatureProvider = Arc<
    dyn Fn(
            FixedBytes<32>,
        ) -> Pin<Box<dyn Future<Output = eyre::Result<Bytes>> + Send>>
        + Send
        + Sync,
>;

#[derive(Clone)]
pub struct Signer {
    pub stub_signature: Bytes,
    pub provider: SignatureProvider,
}

pub fn create_eoa_signer(
    private_key_hex: String,
    eoa_validator_address: Address,
) -> eyre::Result<Signer> {
    let stub_sig = stub_signature_eoa(eoa_validator_address)?;

    let private_key_hex_clone = private_key_hex.clone();
    let eoa_validator_address_clone = eoa_validator_address;

    let signature_provider =
        Arc::new(
            move |hash: FixedBytes<32>| -> Pin<
                Box<dyn Future<Output = eyre::Result<Bytes>> + Send>,
            > {
                let private_key_hex = private_key_hex_clone.clone();
                Box::pin(async move {
                    eoa_signature(
                        &private_key_hex,
                        eoa_validator_address_clone,
                        hash,
                    )
                })
                    as Pin<Box<dyn Future<Output = eyre::Result<Bytes>> + Send>>
            },
        );

    Ok(Signer { stub_signature: stub_sig, provider: signature_provider })
}
