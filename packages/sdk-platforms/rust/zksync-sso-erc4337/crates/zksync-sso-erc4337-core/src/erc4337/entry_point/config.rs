use crate::{
    chain::id::ChainId, erc4337::entry_point::version::EntryPointVersion,
};
use alloy::primitives::{Address, address};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryPointConfig {
    pub chain_id: ChainId,
    pub version: EntryPointVersion,
    pub address: Address,
}

impl Default for EntryPointConfig {
    fn default() -> Self {
        Self {
            chain_id: ChainId::ETHEREUM_MAINNET,
            version: EntryPointVersion::V08,
            address: address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108"),
        }
    }
}
