use crate::{error::ZkSyncSsoError, result::Result};
use alloy::primitives::Address;

#[derive(Debug, Clone)]
pub struct Contracts {
    pub entry_point: Address,
    pub account_factory: Address,
    pub webauthn_validator: Address,
    pub eoa_validator: Address,
}

impl Contracts {
    pub fn new(
        entry_point: Address,
        account_factory: Address,
        webauthn_validator: Address,
        eoa_validator: Address,
    ) -> Self {
        Self { entry_point, account_factory, webauthn_validator, eoa_validator }
    }

    pub fn from_string(
        entry_point: String,
        account_factory: String,
        webauthn_validator: String,
        eoa_validator: String,
    ) -> Result<Self> {
        Ok(Self::new(
            entry_point.parse::<Address>().map_err(|e| {
                ZkSyncSsoError::InvalidConfiguration(e.to_string())
            })?,
            account_factory.parse::<Address>().map_err(|e| {
                ZkSyncSsoError::InvalidConfiguration(e.to_string())
            })?,
            webauthn_validator.parse::<Address>().map_err(|e| {
                ZkSyncSsoError::InvalidConfiguration(e.to_string())
            })?,
            eoa_validator.parse::<Address>().map_err(|e| {
                ZkSyncSsoError::InvalidConfiguration(e.to_string())
            })?,
        ))
    }
}
