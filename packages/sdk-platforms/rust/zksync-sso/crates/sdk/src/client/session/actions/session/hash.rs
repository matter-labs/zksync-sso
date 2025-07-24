use crate::utils::{
    encoding::session::encode_session_key_module_parameters,
    session::session_lib::session_spec::SessionSpec,
};
use alloy::{
    hex::{FromHex, FromHexError},
    primitives::{FixedBytes, keccak256},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionHash(FixedBytes<32>);

impl SessionHash {
    pub fn bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    pub fn fixed_bytes(&self) -> FixedBytes<32> {
        self.0
    }
}

impl From<FixedBytes<32>> for SessionHash {
    fn from(hash: FixedBytes<32>) -> Self {
        SessionHash(hash)
    }
}

impl From<SessionHash> for FixedBytes<32> {
    fn from(val: SessionHash) -> Self {
        val.0
    }
}

impl FromHex for SessionHash {
    type Error = FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let hash = FixedBytes::from_hex(hex)?;
        Ok(SessionHash(hash))
    }
}

pub fn get_session_hash(
    session_spec: SessionSpec,
) -> eyre::Result<SessionHash> {
    let encoded_session = encode_session_key_module_parameters(session_spec)?;
    let session_hash = keccak256(encoded_session);
    Ok(SessionHash(session_hash))
}
