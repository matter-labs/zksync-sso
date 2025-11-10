pub mod active;
pub mod add;
pub mod remove;

use crate::erc4337::account::modular_smart_account::signers::STUB_PRIVATE_KEY;
use alloy::{
    primitives::{Address, Bytes, FixedBytes},
    signers::{SignerSync, local::PrivateKeySigner},
    sol,
};
use std::str::FromStr;

sol!(
    #[sol(rpc)]
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    EOAKeyValidator,
    "../../../../../../packages/erc4337-contracts/out/EOAKeyValidator.sol/EOAKeyValidator.json"
);

pub fn stub_signature_eoa(eoa_validator: Address) -> eyre::Result<Bytes> {
    let hash = FixedBytes::default();
    let private_key_hex = STUB_PRIVATE_KEY;
    let signature = eoa_signature(private_key_hex, eoa_validator, hash)?;
    Ok(signature)
}

pub fn eoa_sign(
    private_key_hex: &str,
    hash: FixedBytes<32>,
) -> eyre::Result<Bytes> {
    let signer = PrivateKeySigner::from_str(private_key_hex)?;
    let signature = signer.sign_hash_sync(&hash)?;
    let signature_bytes = signature.as_bytes();
    Ok(signature_bytes.into())
}

pub fn eoa_signature(
    private_key_hex: &str,
    eoa_validator: Address,
    hash: FixedBytes<32>,
) -> eyre::Result<Bytes> {
    let eoa_validator_bytes = eoa_validator.0.to_vec();
    let signature_bytes = eoa_sign(private_key_hex, hash)?;
    let mut result = vec![0u8; 85];
    result[0..20].copy_from_slice(&eoa_validator_bytes);
    result[20..].copy_from_slice(&signature_bytes);
    let bytes = Bytes::from(result);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    const EOA_VALIDATOR: Address =
        address!("0x00427edf0c3c3bd42188ab4c907759942abebd93");

    #[test]
    fn test_eoa_signer() -> eyre::Result<()> {
        let private_key_hex = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";
        let eoa_validator = EOA_VALIDATOR;

        let hash_hex = "0xfdd7c53bee7cc01a96d3769509d15e568137b6a4b1a56b156bffabf7c510ad06";
        let hash = FixedBytes::from_str(hash_hex)?;

        let expected_signature_hex = "0x00427edf0c3c3bd42188ab4c907759942abebd93eeb7fc6f331132b807e452477a34e4d4106d17e77d8d0a76da66941b2b2fcc7c05b06eeffc84785ba872502f698c2d3e90d1cbddea31c98013145dcf7ccbb22d1c";
        let expected_signature = Bytes::from_str(expected_signature_hex)?;
        let signature = eoa_signature(private_key_hex, eoa_validator, hash)?;

        eyre::ensure!(
            signature == expected_signature,
            "Signature mismatch, received: {signature:?}, expected: {expected_signature:?}"
        );

        Ok(())
    }

    #[test]
    fn test_stub_signature() -> eyre::Result<()> {
        let eoa_validator = EOA_VALIDATOR;

        let signature = stub_signature_eoa(eoa_validator)?;

        let expected_stub_signature_hex = "0x00427edf0c3c3bd42188ab4c907759942abebd9345fc36e56c77a4ff2f9032d5346697bb6f71faccf6b2ce61f5511ad84db29ab20b72aec01a6bbc248622d6622855eb0561063f8ea99fca314bff4359697138d31c";
        let expected_stub_signature =
            Bytes::from_str(expected_stub_signature_hex)?;

        eyre::ensure!(
            signature == expected_stub_signature,
            "Signature mismatch, received: {signature:?}, expected: {expected_stub_signature:?}"
        );

        Ok(())
    }
}
