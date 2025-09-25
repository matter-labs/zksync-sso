use crate::{error::ZkSyncSsoError, result::Result};
use alloy::primitives::Address;

#[derive(Debug, Clone)]
pub struct Contracts {
    pub entry_point: Address,
    pub account_factory: Address,
}

impl Contracts {
    pub fn new(entry_point: Address, account_factory: Address) -> Self {
        Self { entry_point, account_factory }
    }

    pub fn from_string(
        entry_point: String,
        account_factory: String,
    ) -> Result<Self> {
        Ok(Self::new(
            entry_point.parse::<Address>().map_err(|e| {
                ZkSyncSsoError::InvalidConfiguration(e.to_string())
            })?,
            account_factory.parse::<Address>().map_err(|e| {
                ZkSyncSsoError::InvalidConfiguration(e.to_string())
            })?,
        ))
    }
}
