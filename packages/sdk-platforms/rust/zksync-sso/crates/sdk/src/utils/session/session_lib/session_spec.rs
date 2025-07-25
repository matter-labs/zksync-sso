use crate::{
    contracts::SessionLib::{
        CallSpec as SessionLibCallSpec, SessionSpec as SessionLibSpec,
        TransferSpec as SessionLibTransferSpec,
        UsageLimit as SessionLibUsageLimit,
    },
    utils::session::session_lib::session_spec::{
        call_spec::CallSpec, transfer_spec::TransferSpec,
        usage_limit::UsageLimit,
    },
};
use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};

pub mod call_spec;
pub mod condition;
pub mod constraint;
pub mod limit_type;
pub mod transfer_spec;
pub mod usage_limit;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionSpec {
    pub signer: Address,
    pub expires_at: U256,
    pub fee_limit: UsageLimit,
    pub call_policies: Vec<CallSpec>,
    pub transfer_policies: Vec<TransferSpec>,
}

impl From<SessionSpec> for SessionLibSpec {
    fn from(session_config: SessionSpec) -> Self {
        let fee_limit: SessionLibUsageLimit = session_config.fee_limit.into();
        let call_policies: Vec<SessionLibCallSpec> =
            session_config.call_policies.into_iter().map(Into::into).collect();
        let transfer_policies: Vec<SessionLibTransferSpec> = session_config
            .transfer_policies
            .into_iter()
            .map(Into::into)
            .collect();
        SessionLibSpec {
            signer: session_config.signer,
            expiresAt: session_config.expires_at,
            feeLimit: fee_limit,
            callPolicies: call_policies,
            transferPolicies: transfer_policies,
        }
    }
}

impl From<SessionLibSpec> for SessionSpec {
    fn from(value: SessionLibSpec) -> Self {
        SessionSpec {
            signer: value.signer,
            expires_at: value.expiresAt,
            fee_limit: value.feeLimit.into(),
            call_policies: value
                .callPolicies
                .into_iter()
                .map(Into::into)
                .collect(),
            transfer_policies: value
                .transferPolicies
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Policy {
    Transfer(TransferSpec),
    Call(CallSpec),
}

impl Policy {
    pub fn usage_limit(&self) -> &UsageLimit {
        match self {
            Policy::Transfer(spec) => &spec.value_limit,
            Policy::Call(spec) => &spec.value_limit,
        }
    }

    pub fn as_call_policy(&self) -> Option<&CallSpec> {
        match self {
            Policy::Call(spec) => Some(spec),
            _ => None,
        }
    }
}

impl From<TransferSpec> for Policy {
    fn from(spec: TransferSpec) -> Self {
        Policy::Transfer(spec)
    }
}

impl From<CallSpec> for Policy {
    fn from(spec: CallSpec) -> Self {
        Policy::Call(spec)
    }
}

impl From<SessionLibTransferSpec> for Policy {
    fn from(spec: SessionLibTransferSpec) -> Self {
        Policy::Transfer(spec.into())
    }
}

impl From<SessionLibCallSpec> for Policy {
    fn from(spec: SessionLibCallSpec) -> Self {
        Policy::Call(spec.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        contracts::SessionLib::{
            SessionSpec as SessionLibSpec, UsageLimit as SessionLibUsageLimit,
        },
        utils::session::session_lib::session_spec::{
            condition::Condition, constraint::Constraint, limit_type::LimitType,
        },
    };
    use alloy::primitives::{FixedBytes, address, fixed_bytes};

    #[test]
    fn test_session_spec_round_trip_conversion() {
        // Create a comprehensive SessionSpec with all fields populated
        let original_session_spec = SessionSpec {
            signer: address!("0x9BbC92a33F193174bf6Cc09c4b4055500d972479"),
            expires_at: U256::from(1749040108u64),
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(100000000000000000u64),
                period: U256::ZERO,
            },
            call_policies: vec![CallSpec {
                target: address!("0x1111111111111111111111111111111111111111"),
                selector: fixed_bytes!("a9059cbb"),
                max_value_per_use: U256::from(1000000000000000u64),
                value_limit: UsageLimit {
                    limit_type: LimitType::Lifetime,
                    limit: U256::from(10000000000000000u64),
                    period: U256::ZERO,
                },
                constraints: vec![Constraint {
                    condition: Condition::Equal,
                    index: 4,
                    ref_value: FixedBytes::ZERO,
                    limit: UsageLimit {
                        limit_type: LimitType::Allowance,
                        limit: U256::from(5000000000000000u64),
                        period: U256::from(1800u64),
                    },
                }],
            }],
            transfer_policies: vec![TransferSpec {
                target: address!("0x2222222222222222222222222222222222222222"),
                max_value_per_use: U256::from(20000000000000000u64),
                value_limit: UsageLimit {
                    limit_type: LimitType::Unlimited,
                    limit: U256::ZERO,
                    period: U256::ZERO,
                },
            }],
        };

        // Convert SessionSpec -> SessionLibSpec -> SessionSpec
        let session_lib_spec: SessionLibSpec =
            original_session_spec.clone().into();
        let round_trip_session_spec: SessionSpec = session_lib_spec.into();

        // Assert that the round-trip conversion preserves all data
        assert_eq!(original_session_spec, round_trip_session_spec);
    }

    #[test]
    fn test_session_spec_round_trip_with_empty_policies() {
        // Test with minimal SessionSpec (empty policies)
        let original_session_spec = SessionSpec {
            signer: address!("0x9BbC92a33F193174bf6Cc09c4b4055500d972479"),
            expires_at: U256::from(1749040108u64),
            fee_limit: UsageLimit::UNLIMITED,
            call_policies: vec![],
            transfer_policies: vec![],
        };

        // Convert SessionSpec -> SessionLibSpec -> SessionSpec
        let session_lib_spec: SessionLibSpec =
            original_session_spec.clone().into();
        let round_trip_session_spec: SessionSpec = session_lib_spec.into();

        // Assert that the round-trip conversion preserves all data
        assert_eq!(original_session_spec, round_trip_session_spec);
    }

    #[test]
    fn test_usage_limit_types_round_trip() {
        let test_cases = vec![
            UsageLimit::UNLIMITED,
            UsageLimit::ZERO,
            UsageLimit {
                limit_type: LimitType::Allowance,
                limit: U256::from(1000u64),
                period: U256::from(3600u64),
            },
        ];

        for original_limit in test_cases {
            let session_lib_limit: SessionLibUsageLimit =
                original_limit.clone().into();
            let round_trip_limit: UsageLimit = session_lib_limit.into();
            assert_eq!(original_limit, round_trip_limit);
        }
    }
}
