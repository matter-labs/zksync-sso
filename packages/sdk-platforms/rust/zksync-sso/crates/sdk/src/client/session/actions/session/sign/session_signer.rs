use alloy::{
    primitives::{Address, Bytes, FixedBytes},
    signers::{SignerSync, local::PrivateKeySigner},
};
use std::str::FromStr;

pub trait SessionSigner {
    #[allow(dead_code)]
    fn address(&self) -> Address;

    fn sign(&self, hash: FixedBytes<32>) -> eyre::Result<Bytes>;
}

pub type SignerBox = Box<dyn SessionSigner>;

pub struct BasicSessionSigner {
    signer: PrivateKeySigner,
}

impl BasicSessionSigner {
    #[allow(dead_code)]
    pub fn new(private_key_hex: String) -> eyre::Result<Self> {
        Self::try_from(private_key_hex)
    }

    #[allow(dead_code)]
    pub fn new_from_bytes(bytes: FixedBytes<32>) -> eyre::Result<Self> {
        Self::try_from(bytes)
    }
}

impl SessionSigner for BasicSessionSigner {
    fn address(&self) -> Address {
        self.signer.address()
    }

    fn sign(&self, hash: FixedBytes<32>) -> eyre::Result<Bytes> {
        let signature = self.signer.sign_hash_sync(&hash)?;
        let signature_array = signature.as_bytes();
        let signature_fixed_bytes: FixedBytes<65> =
            FixedBytes::from_slice(signature_array.as_slice());
        Ok(signature_fixed_bytes.into())
    }
}

impl TryFrom<String> for BasicSessionSigner {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let signer = PrivateKeySigner::from_str(&value)?;
        Ok(Self { signer })
    }
}

impl TryFrom<FixedBytes<32>> for BasicSessionSigner {
    type Error = eyre::Error;

    fn try_from(value: FixedBytes<32>) -> Result<Self, Self::Error> {
        let signer = PrivateKeySigner::from_bytes(&value)?;
        Ok(Self { signer })
    }
}

pub fn private_key_to_signer(
    private_key: FixedBytes<32>,
) -> eyre::Result<SignerBox> {
    Ok(Box::new(BasicSessionSigner::try_from(private_key)?))
}
