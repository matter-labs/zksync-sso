use alloy::primitives::B256;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// User operation hash
#[derive(
    Eq,
    Hash,
    PartialEq,
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Default,
    PartialOrd,
    Ord,
)]
pub struct UserOperationHash(pub B256);

impl From<B256> for UserOperationHash {
    fn from(value: B256) -> Self {
        Self(B256::from_slice(&value.0))
    }
}

impl From<UserOperationHash> for B256 {
    fn from(value: UserOperationHash) -> Self {
        B256::from_slice(&value.0.0)
    }
}

impl From<[u8; 32]> for UserOperationHash {
    fn from(value: [u8; 32]) -> Self {
        Self(B256::from_slice(&value))
    }
}

impl FromStr for UserOperationHash {
    type Err = alloy::hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        B256::from_str(s).map(|h| h.into())
    }
}

impl UserOperationHash {
    #[inline]
    pub const fn as_fixed_bytes(&self) -> &[u8; 32] {
        &self.0.0
    }

    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0.0
    }

    #[inline]
    pub fn repeat_byte(byte: u8) -> UserOperationHash {
        UserOperationHash(B256::from_slice(&[byte; 32]))
    }

    #[inline]
    pub fn zero() -> UserOperationHash {
        UserOperationHash::repeat_byte(0u8)
    }

    pub fn assign_from_slice(&mut self, src: &[u8]) {
        self.as_bytes_mut().copy_from_slice(src);
    }

    pub fn from_slice(src: &[u8]) -> Self {
        let mut ret = Self::zero();
        ret.assign_from_slice(src);
        ret
    }
}
