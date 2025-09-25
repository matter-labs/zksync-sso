use crate::{
    chain::id::ChainId, erc4337::entry_point::version::EntryPointVersion,
};
use alloy::primitives::Address;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryPointConfig {
    pub chain_id: ChainId,
    pub version: EntryPointVersion,
    pub address: Address,
}
