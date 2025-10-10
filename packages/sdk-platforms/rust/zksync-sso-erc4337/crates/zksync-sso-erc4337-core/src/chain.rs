pub mod id;

use crate::{
    chain::id::ChainId, erc4337::entry_point::version::EntryPointVersion,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Chain {
    pub id: ChainId,
    pub entry_point_version: EntryPointVersion,
    pub name: String,
}

impl Chain {
    pub fn new(
        id: ChainId,
        entry_point_version: EntryPointVersion,
        name: String,
    ) -> Self {
        Self { id, entry_point_version, name }
    }
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
            name: "".to_string(),
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}
