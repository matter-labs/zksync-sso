use alloy::primitives::Address;
use eyre::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PasskeyContracts {
    pub account_factory: Address,
    pub passkey: Address,
    pub session: Address,
    pub account_paymaster: Address,
}

impl PasskeyContracts {
    pub fn new(
        account_factory: Address,
        passkey: Address,
        session: Address,
        account_paymaster: Address,
    ) -> Self {
        Self { account_factory, passkey, session, account_paymaster }
    }

    pub fn with_address_strs(
        account_factory: &str,
        passkey: &str,
        session: &str,
        account_paymaster: &str,
    ) -> Result<Self> {
        Ok(Self {
            account_factory: account_factory.parse().map_err(|e| {
                eyre::eyre!("Invalid account factory address: {}", e)
            })?,
            passkey: passkey
                .parse()
                .map_err(|e| eyre::eyre!("Invalid passkey address: {}", e))?,
            session: session
                .parse()
                .map_err(|e| eyre::eyre!("Invalid session address: {}", e))?,
            account_paymaster: account_paymaster.parse().map_err(|e| {
                eyre::eyre!("Invalid account paymaster address: {}", e)
            })?,
        })
    }
}
