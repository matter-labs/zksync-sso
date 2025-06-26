use alloy::primitives::Address;
use eyre::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyContracts {
    pub account_factory: Address,
    pub passkey: Address,
    pub session: Address,
    pub account_paymaster: Address,
    pub recovery: Address,
}

impl PasskeyContracts {
    pub fn new(
        account_factory: Address,
        passkey: Address,
        session: Address,
        account_paymaster: Address,
        recovery: Address,
    ) -> Self {
        Self { account_factory, passkey, session, account_paymaster, recovery }
    }

    pub fn with_address_strs(
        account_factory: &str,
        passkey: &str,
        session: &str,
        account_paymaster: &str,
        recovery: &str,
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
            recovery: recovery
                .parse()
                .map_err(|e| eyre::eyre!("Invalid recovery address: {}", e))?,
        })
    }
}
