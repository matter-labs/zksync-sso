use crate::{
    contracts::SessionLib::Constraint as SessionLibConstraint,
    utils::session::session_lib::session_spec::{
        condition::Condition, usage_limit::UsageLimit,
    },
};
use alloy::primitives::FixedBytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Constraint {
    pub condition: Condition,
    pub index: u64,
    pub ref_value: FixedBytes<32>,
    pub limit: UsageLimit,
}

impl From<SessionLibConstraint> for Constraint {
    fn from(value: SessionLibConstraint) -> Self {
        Constraint {
            condition: value.condition.try_into().unwrap(),
            index: value.index,
            ref_value: value.refValue,
            limit: value.limit.into(),
        }
    }
}

impl From<Constraint> for SessionLibConstraint {
    fn from(value: Constraint) -> Self {
        SessionLibConstraint {
            condition: value.condition.into(),
            index: value.index,
            refValue: value.ref_value,
            limit: value.limit.into(),
        }
    }
}
