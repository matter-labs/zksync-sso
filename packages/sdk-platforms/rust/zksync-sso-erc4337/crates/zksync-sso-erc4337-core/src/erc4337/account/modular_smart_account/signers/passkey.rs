pub mod active;
pub mod add;
pub mod encoding;
pub mod remove;

use crate::erc4337::account::modular_smart_account::signers::{
    STUB_PRIVATE_KEY, eoa::eoa_signature,
};
use alloy::primitives::{Address, Bytes, FixedBytes};

pub fn stub_signature_passkey(eoa_validator: Address) -> eyre::Result<Bytes> {
    let hash = FixedBytes::default();
    let private_key_hex = STUB_PRIVATE_KEY;
    let signature = eoa_signature(private_key_hex, eoa_validator, hash)?;
    Ok(signature)
}
