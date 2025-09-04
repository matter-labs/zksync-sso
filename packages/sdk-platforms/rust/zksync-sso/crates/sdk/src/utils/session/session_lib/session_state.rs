use crate::{
    contracts::SessionLib::SessionState as SessionLibSessionState,
    utils::{
        alloy::serde_helpers::{
            deserialize_u256_from_integer_string,
            serialize_u256_as_integer_string,
        },
        session::session_lib::session_state::{
            limit_state::LimitState, status::Status,
        },
    },
};
use alloy::primitives::U256;
use serde::{Deserialize, Serialize};

pub mod limit_state;
pub mod status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    pub status: Status,
    #[serde(
        serialize_with = "serialize_u256_as_integer_string",
        deserialize_with = "deserialize_u256_from_integer_string"
    )]
    pub fees_remaining: U256,
    pub transfer_value: Vec<LimitState>,
    pub call_value: Vec<LimitState>,
    pub call_params: Vec<LimitState>,
}

impl SessionState {
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    pub fn is_closed(&self) -> bool {
        self.status.is_closed()
    }
}

impl From<SessionLibSessionState> for SessionState {
    fn from(value: SessionLibSessionState) -> Self {
        SessionState {
            status: value.status.try_into().unwrap(),
            fees_remaining: value.feesRemaining,
            transfer_value: value
                .transferValue
                .into_iter()
                .map(|x| x.into())
                .collect(),
            call_value: value.callValue.into_iter().map(|x| x.into()).collect(),
            call_params: value
                .callParams
                .into_iter()
                .map(|x| x.into())
                .collect(),
        }
    }
}
