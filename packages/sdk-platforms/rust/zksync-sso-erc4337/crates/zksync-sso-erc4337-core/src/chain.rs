use crate::{
    chain::id::ChainId, erc4337::entry_point::version::EntryPointVersion,
};
use std::fmt;

pub mod id;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Chain {
    pub id: ChainId,
    pub entry_point_version: EntryPointVersion,
    pub name: &'static str,
}

impl Chain {
    pub fn new(
        id: ChainId,
        entry_point_version: EntryPointVersion,
        name: &'static str,
    ) -> Self {
        Self { id, entry_point_version, name }
    }

    pub const ETHEREUM_MAINNET_V07: Self = Self {
        id: ChainId::ETHEREUM_MAINNET,
        entry_point_version: EntryPointVersion::V07,
        name: "Ethereum Mainnet",
    };

    pub const ETHEREUM_SEPOLIA_V07: Self = Self {
        id: ChainId::ETHEREUM_SEPOLIA,
        entry_point_version: EntryPointVersion::V07,
        name: "Ethereum Sepolia",
    };

    pub const BASE_SEPOLIA_V07: Self = Self {
        id: ChainId::BASE_SEPOLIA,
        entry_point_version: EntryPointVersion::V07,
        name: "Base Sepolia",
    };

    pub const LOCAL_ETHEREUM_SEPOLIA_V07: Self = Self {
        id: ChainId::LOCAL_FOUNDRY_ETHEREUM_SEPOLIA,
        entry_point_version: EntryPointVersion::V07,
        name: "Local Ethereum Sepolia",
    };
}

impl Chain {
    pub fn caip2_identifier(&self) -> String {
        self.id.caip2_identifier()
    }
}

impl From<ChainId> for Chain {
    fn from(chain_id: ChainId) -> Self {
        Self {
            id: chain_id,
            entry_point_version: EntryPointVersion::V07,
            name: "",
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}
