use crate::{
    erc4337::account::modular_smart_account::session::SessionLib::LimitState as SessionLibLimitState,
    utils::alloy_utilities::serde_helpers::{
        deserialize_u256_from_integer_string, serialize_u256_as_integer_string,
    },
};
use alloy::primitives::{Address, FixedBytes, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LimitState {
    #[serde(
        serialize_with = "serialize_u256_as_integer_string",
        deserialize_with = "deserialize_u256_from_integer_string"
    )]
    pub remaining: U256,
    pub target: Address,
    pub selector: FixedBytes<4>,
    #[serde(
        serialize_with = "serialize_u256_as_integer_string",
        deserialize_with = "deserialize_u256_from_integer_string"
    )]
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
