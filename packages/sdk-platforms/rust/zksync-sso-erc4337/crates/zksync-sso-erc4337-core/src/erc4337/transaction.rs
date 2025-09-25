use alloy::primitives::{Address, Bytes, U256};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub to: Address,
    pub value: U256,
    pub data: Bytes,
}
