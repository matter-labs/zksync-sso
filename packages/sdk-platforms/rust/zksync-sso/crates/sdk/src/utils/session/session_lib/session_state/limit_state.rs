use crate::contracts::SessionLib::LimitState as SessionLibLimitState;
use alloy::primitives::{Address, FixedBytes, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LimitState {
    pub remaining: U256,
    pub target: Address,
    pub selector: FixedBytes<4>,
    pub index: U256,
}

impl From<SessionLibLimitState> for LimitState {
    fn from(value: SessionLibLimitState) -> Self {
        LimitState {
            remaining: value.remaining,
            target: value.target,
            selector: value.selector,
            index: value.index,
        }
    }
}
