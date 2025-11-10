use crate::erc4337::account::modular_smart_account::session::{
    contract::SessionLib::CallSpec as SessionLibCallSpec,
    session_lib::session_spec::{
        constraint::Constraint, usage_limit::UsageLimit,
    },
};
use alloy::primitives::{Address, FixedBytes, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CallSpec {
    pub target: Address,
    pub selector: FixedBytes<4>,
    pub max_value_per_use: U256,
    pub value_limit: UsageLimit,
    pub constraints: Vec<Constraint>,
}

impl From<SessionLibCallSpec> for CallSpec {
    fn from(value: SessionLibCallSpec) -> Self {
        CallSpec {
            target: value.target,
            selector: value.selector,
            max_value_per_use: value.maxValuePerUse,
            value_limit: value.valueLimit.into(),
            constraints: value
                .constraints
                .into_iter()
                .map(|x| x.into())
                .collect(),
        }
    }
}

impl From<CallSpec> for SessionLibCallSpec {
    fn from(value: CallSpec) -> Self {
        SessionLibCallSpec {
            target: value.target,
            selector: value.selector,
            maxValuePerUse: value.max_value_per_use,
            valueLimit: value.value_limit.into(),
            constraints: value
                .constraints
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::account::modular_smart_account::session::{
        contract::SessionLib::{
            Constraint as SessionLibConstraint,
            UsageLimit as SessionLibUsageLimit,
        },
        session_lib::session_spec::{
            condition::Condition, constraint::Constraint,
            limit_type::LimitType, usage_limit::UsageLimit,
        },
    };
    use alloy::primitives::{
        FixedBytes, U256, address, aliases::U48, fixed_bytes,
    };

    #[test]
    fn test_call_spec_round_trip_conversions() {
        let test_cases = vec![
            // Test case 1: Minimal CallSpec with no constraints
            (
                address!("0x1111111111111111111111111111111111111111"),
                fixed_bytes!("0x12345678"),
                U256::ZERO,
                UsageLimit {
                    limit_type: LimitType::Unlimited,
                    limit: U256::ZERO,
                    period: U48::ZERO,
                },
                vec![],
                "Minimal CallSpec with no constraints",
            ),
            // Test case 2: CallSpec with single constraint
            (
                address!("0x2222222222222222222222222222222222222222"),
                fixed_bytes!("0xa9059cbb"),
                U256::from(1000u64),
                UsageLimit {
                    limit_type: LimitType::Lifetime,
                    limit: U256::from(5000u64),
                    period: U48::from(3600u32),
                },
                vec![Constraint {
                    condition: Condition::Equal,
                    index: 0,
                    ref_value: FixedBytes::from([42u8; 32]),
                    limit: UsageLimit::UNLIMITED,
                }],
                "CallSpec with single constraint",
            ),
            // Test case 3: CallSpec with multiple constraints
            (
                address!("0x3333333333333333333333333333333333333333"),
                fixed_bytes!("0x23b872dd"),
                U256::from(500_000u64),
                UsageLimit {
                    limit_type: LimitType::Allowance,
                    limit: U256::from(1_000_000u64),
                    period: U48::from(86400u32),
                },
                vec![
                    Constraint {
                        condition: Condition::Greater,
                        index: 0,
                        ref_value: FixedBytes::from([100u8; 32]),
                        limit: UsageLimit::UNLIMITED,
                    },
                    Constraint {
                        condition: Condition::LessEqual,
                        index: 1,
                        ref_value: FixedBytes::from([200u8; 32]),
                        limit: UsageLimit::UNLIMITED,
                    },
                    Constraint {
                        condition: Condition::NotEqual,
                        index: 2,
                        ref_value: FixedBytes::ZERO,
                        limit: UsageLimit::UNLIMITED,
                    },
                ],
                "CallSpec with multiple constraints",
            ),
            // Test case 4: Maximum values
            (
                address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),
                fixed_bytes!("0xFFFFFFFF"),
                U256::MAX,
                UsageLimit {
                    limit_type: LimitType::Lifetime,
                    limit: U256::MAX,
                    period: U48::MAX,
                },
                vec![Constraint {
                    index: 255,
                    condition: Condition::Unconstrained,
                    ref_value: FixedBytes::from([0u8; 32]),
                    limit: UsageLimit::UNLIMITED,
                }],
                "Maximum values with single constraint",
            ),
            // Test case 5: Edge case with all condition types
            (
                address!("0xDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEF"),
                fixed_bytes!("0xdeadbeef"),
                U256::from(12345u64),
                UsageLimit {
                    limit_type: LimitType::Allowance,
                    limit: U256::from(67890u64),
                    period: U48::ZERO,
                },
                vec![
                    Constraint {
                        index: 0,
                        condition: Condition::Unconstrained,
                        ref_value: FixedBytes::from([1u8; 32]),
                        limit: UsageLimit::UNLIMITED,
                    },
                    Constraint {
                        index: 1,
                        condition: Condition::Equal,
                        ref_value: FixedBytes::from([2u8; 32]),
                        limit: UsageLimit::UNLIMITED,
                    },
                    Constraint {
                        index: 2,
                        condition: Condition::Greater,
                        ref_value: FixedBytes::from([3u8; 32]),
                        limit: UsageLimit::UNLIMITED,
                    },
                    Constraint {
                        index: 3,
                        condition: Condition::Less,
                        ref_value: FixedBytes::from([4u8; 32]),
                        limit: UsageLimit::UNLIMITED,
                    },
                    Constraint {
                        index: 4,
                        condition: Condition::GreaterEqual,
                        ref_value: FixedBytes::from([5u8; 32]),
                        limit: UsageLimit::UNLIMITED,
                    },
                    Constraint {
                        index: 5,
                        condition: Condition::LessEqual,
                        ref_value: FixedBytes::from([6u8; 32]),
                        limit: UsageLimit::UNLIMITED,
                    },
                    Constraint {
                        index: 6,
                        condition: Condition::NotEqual,
                        ref_value: FixedBytes::from([7u8; 32]),
                        limit: UsageLimit::UNLIMITED,
                    },
                ],
                "Edge case with all condition types",
            ),
        ];

        for (
            target,
            selector,
            max_value_per_use,
            value_limit,
            constraints,
            description,
        ) in test_cases
        {
            // Create original SessionLibCallSpec
            let original_session_lib = SessionLibCallSpec {
                target,
                selector,
                maxValuePerUse: max_value_per_use,
                valueLimit: SessionLibUsageLimit {
                    limitType: value_limit.limit_type.clone().into(),
                    limit: value_limit.limit,
                    period: value_limit.period,
                },
                constraints: constraints
                    .iter()
                    .map(|c| SessionLibConstraint {
                        index: c.clone().index,
                        condition: c.clone().condition.into(),
                        refValue: c.clone().ref_value,
                        limit: c.clone().limit.into(),
                    })
                    .collect(),
            };

            // Convert SessionLibCallSpec -> CallSpec
            let call_spec: CallSpec = original_session_lib.clone().into();

            // Verify the conversion preserved all values
            assert_eq!(
                call_spec.target, target,
                "Target address mismatch for test case: {description}"
            );
            assert_eq!(
                call_spec.selector, selector,
                "Selector mismatch for test case: {description}"
            );
            assert_eq!(
                call_spec.max_value_per_use, max_value_per_use,
                "Max value per use mismatch for test case: {description}"
            );
            assert_eq!(
                call_spec.value_limit, value_limit,
                "Value limit mismatch for test case: {description}"
            );
            assert_eq!(
                call_spec.constraints, constraints,
                "Constraints mismatch for test case: {description}"
            );

            // Convert CallSpec -> SessionLibCallSpec (round-trip)
            let round_trip_session_lib: SessionLibCallSpec =
                call_spec.clone().into();

            // Verify the round-trip preserved all values
            assert_eq!(
                round_trip_session_lib.target, original_session_lib.target,
                "Round-trip target address mismatch for test case: {description}"
            );
            assert_eq!(
                round_trip_session_lib.selector, original_session_lib.selector,
                "Round-trip selector mismatch for test case: {description}"
            );
            assert_eq!(
                round_trip_session_lib.maxValuePerUse,
                original_session_lib.maxValuePerUse,
                "Round-trip max value per use mismatch for test case: {description}"
            );
            assert_eq!(
                round_trip_session_lib.valueLimit.limitType,
                original_session_lib.valueLimit.limitType,
                "Round-trip value limit type mismatch for test case: {description}"
            );
            assert_eq!(
                round_trip_session_lib.valueLimit.limit,
                original_session_lib.valueLimit.limit,
                "Round-trip value limit amount mismatch for test case: {description}"
            );
            assert_eq!(
                round_trip_session_lib.valueLimit.period,
                original_session_lib.valueLimit.period,
                "Round-trip value limit period mismatch for test case: {description}"
            );
            assert_eq!(
                round_trip_session_lib.constraints.len(),
                original_session_lib.constraints.len(),
                "Round-trip constraints length mismatch for test case: {description}"
            );

            // Verify each constraint individually
            for (i, (round_trip_constraint, original_constraint)) in
                round_trip_session_lib
                    .constraints
                    .iter()
                    .zip(original_session_lib.constraints.iter())
                    .enumerate()
            {
                assert_eq!(
                    round_trip_constraint.index, original_constraint.index,
                    "Round-trip constraint {i} index mismatch for test case: {description}"
                );
                assert_eq!(
                    round_trip_constraint.condition,
                    original_constraint.condition,
                    "Round-trip constraint {i} condition mismatch for test case: {description}"
                );
                assert_eq!(
                    round_trip_constraint.refValue,
                    original_constraint.refValue,
                    "Round-trip constraint {i} reference value mismatch for test case: {description}"
                );
            }

            // Additional verification: convert back to CallSpec again
            let double_round_trip: CallSpec = round_trip_session_lib.into();

            assert_eq!(
                double_round_trip.target, call_spec.target,
                "Double round-trip target mismatch for test case: {description}"
            );
            assert_eq!(
                double_round_trip.selector, call_spec.selector,
                "Double round-trip selector mismatch for test case: {description}"
            );
            assert_eq!(
                double_round_trip.max_value_per_use,
                call_spec.max_value_per_use,
                "Double round-trip max value per use mismatch for test case: {description}"
            );
            assert_eq!(
                double_round_trip.value_limit, call_spec.value_limit,
                "Double round-trip value limit mismatch for test case: {description}"
            );
            assert_eq!(
                double_round_trip.constraints, call_spec.constraints,
                "Double round-trip constraints mismatch for test case: {description}"
            );
        }
    }

    #[test]
    fn test_call_spec_field_preservation() {
        // Test that each field is independently preserved during conversion
        let original = CallSpec {
            target: address!("0x4242424242424242424242424242424242424242"),
            selector: fixed_bytes!("0x12345678"),
            max_value_per_use: U256::from(999u64),
            value_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(1234u64),
                period: U48::from(5678u32),
            },
            constraints: vec![
                Constraint {
                    index: 10,
                    condition: Condition::Equal,
                    ref_value: FixedBytes::from([100u8; 32]),
                    limit: UsageLimit::UNLIMITED,
                },
                Constraint {
                    index: 20,
                    condition: Condition::Greater,
                    ref_value: FixedBytes::from([200u8; 32]),
                    limit: UsageLimit::UNLIMITED,
                },
            ],
        };

        // Convert to SessionLibCallSpec
        let session_lib: SessionLibCallSpec = original.clone().into();

        // Verify each field individually
        assert_eq!(session_lib.target, original.target);
        assert_eq!(session_lib.selector, original.selector);
        assert_eq!(session_lib.maxValuePerUse, original.max_value_per_use);

        // Verify nested UsageLimit fields
        assert_eq!(session_lib.valueLimit.limit, original.value_limit.limit);
        assert_eq!(session_lib.valueLimit.period, original.value_limit.period);

        // Verify limit type numeric value
        let limit_type_numeric: u8 = session_lib.valueLimit.limitType;
        let expected_numeric: u8 =
            original.value_limit.limit_type.clone().into();
        assert_eq!(limit_type_numeric, expected_numeric);

        // Verify constraints
        assert_eq!(session_lib.constraints.len(), original.constraints.len());
        for (session_constraint, original_constraint) in
            session_lib.constraints.iter().zip(original.constraints.iter())
        {
            assert_eq!(session_constraint.index, original_constraint.index);
            assert_eq!(
                session_constraint.refValue,
                original_constraint.ref_value
            );

            let condition_numeric: u8 = session_constraint.condition;
            let expected_condition_numeric: u8 =
                original_constraint.condition.clone().into();
            assert_eq!(condition_numeric, expected_condition_numeric);
        }

        // Convert back and verify complete equality
        let round_trip: CallSpec = session_lib.into();
        assert_eq!(round_trip, original);
    }

    #[test]
    fn test_call_spec_with_all_limit_types() {
        let base_target =
            address!("0x3333333333333333333333333333333333333333");
        let base_selector = fixed_bytes!("0xa9059cbb");
        let base_max_value = U256::from(777u64);
        let base_limit = U256::from(888u64);
        let base_period = U48::from(999u32);
        let base_constraints = vec![Constraint {
            index: 1,
            condition: Condition::Equal,
            ref_value: FixedBytes::from([123u8; 32]),
            limit: UsageLimit::UNLIMITED,
        }];

        let limit_types = vec![
            (LimitType::Unlimited, 0u8),
            (LimitType::Lifetime, 1u8),
            (LimitType::Allowance, 2u8),
        ];

        for (limit_type, expected_numeric) in limit_types {
            let call_spec = CallSpec {
                target: base_target,
                selector: base_selector,
                max_value_per_use: base_max_value,
                value_limit: UsageLimit {
                    limit_type: limit_type.clone(),
                    limit: base_limit,
                    period: base_period,
                },
                constraints: base_constraints.clone(),
            };

            // Convert to SessionLibCallSpec
            let session_lib: SessionLibCallSpec = call_spec.clone().into();

            // Verify limit type numeric value
            let numeric_value: u8 = session_lib.valueLimit.limitType;
            assert_eq!(
                numeric_value, expected_numeric,
                "Numeric value mismatch for limit type: {limit_type:?}"
            );

            // Convert back and verify equality
            let back_to_call_spec: CallSpec = session_lib.into();
            assert_eq!(back_to_call_spec, call_spec);
        }
    }

    #[test]
    fn test_call_spec_equality_after_conversions() {
        let original = CallSpec {
            target: address!("0xDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEF"),
            selector: fixed_bytes!("0xdeadbeef"),
            max_value_per_use: U256::from(12345u64),
            value_limit: UsageLimit {
                limit_type: LimitType::Allowance,
                limit: U256::from(67890u64),
                period: U48::from(11111u32),
            },
            constraints: vec![
                Constraint {
                    index: 0,
                    condition: Condition::Greater,
                    ref_value: FixedBytes::from([5u8; 32]),
                    limit: UsageLimit::UNLIMITED,
                },
                Constraint {
                    index: 1,
                    condition: Condition::LessEqual,
                    ref_value: FixedBytes::from([2u8; 32]),
                    limit: UsageLimit::UNLIMITED,
                },
            ],
        };

        // Multiple conversion paths should all result in equivalent objects
        let path1 = {
            let session_lib: SessionLibCallSpec = original.clone().into();
            let back: CallSpec = session_lib.into();
            back
        };

        let path2 = {
            let session_lib: SessionLibCallSpec = original.clone().into();
            let intermediate: CallSpec = session_lib.into();
            let session_lib2: SessionLibCallSpec = intermediate.into();
            let back: CallSpec = session_lib2.into();
            back
        };

        assert_eq!(original, path1);
        assert_eq!(original, path2);
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_call_spec_selector_variations() {
        // Test various function selector patterns
        let selector_patterns = vec![
            (fixed_bytes!("0x00000000"), "Zero selector"),
            (fixed_bytes!("0xa9059cbb"), "transfer(address,uint256)"),
            (
                fixed_bytes!("0x23b872dd"),
                "transferFrom(address,address,uint256)",
            ),
            (fixed_bytes!("0x095ea7b3"), "approve(address,uint256)"),
            (fixed_bytes!("0xFFFFFFFF"), "Max selector"),
            (fixed_bytes!("0x12345678"), "Custom selector"),
        ];

        for (selector, description) in selector_patterns {
            let call_spec = CallSpec {
                target: address!("0x1111111111111111111111111111111111111111"),
                selector,
                max_value_per_use: U256::from(100u64),
                value_limit: UsageLimit::UNLIMITED,
                constraints: vec![],
            };

            // Test round-trip conversion
            let session_lib: SessionLibCallSpec = call_spec.clone().into();
            let back_to_call_spec: CallSpec = session_lib.into();

            assert_eq!(
                back_to_call_spec, call_spec,
                "Round-trip failed for selector case: {description}"
            );
            assert_eq!(
                back_to_call_spec.selector, selector,
                "Selector not preserved for case: {description}"
            );
        }
    }

    #[test]
    fn test_call_spec_constraint_edge_cases() {
        // Test various constraint combinations
        let constraint_cases = vec![
            (vec![], "No constraints"),
            (
                vec![Constraint {
                    index: 0,
                    condition: Condition::Unconstrained,
                    ref_value: FixedBytes::from([0u8; 32]),
                    limit: UsageLimit::UNLIMITED,
                }],
                "Single unconstrained",
            ),
            (
                vec![
                    Constraint {
                        index: 0,
                        condition: Condition::Equal,
                        ref_value: FixedBytes::from([42u8; 32]),
                        limit: UsageLimit::UNLIMITED,
                    },
                    Constraint {
                        index: 1,
                        condition: Condition::NotEqual,
                        ref_value: FixedBytes::from([0u8; 32]),
                        limit: UsageLimit::UNLIMITED,
                    },
                ],
                "Two constraints",
            ),
            (
                vec![Constraint {
                    index: 255,
                    condition: Condition::Greater,
                    ref_value: FixedBytes::from([0u8; 32]),
                    limit: UsageLimit::UNLIMITED,
                }],
                "Max index and value",
            ),
        ];

        for (constraints, description) in constraint_cases {
            let call_spec = CallSpec {
                target: address!("0x2222222222222222222222222222222222222222"),
                selector: fixed_bytes!("0xa9059cbb"),
                max_value_per_use: U256::from(1000u64),
                value_limit: UsageLimit::UNLIMITED,
                constraints: constraints.clone(),
            };

            // Test round-trip conversion
            let session_lib: SessionLibCallSpec = call_spec.clone().into();
            let back_to_call_spec: CallSpec = session_lib.into();

            assert_eq!(
                back_to_call_spec, call_spec,
                "Round-trip failed for constraint case: {description}"
            );
            assert_eq!(
                back_to_call_spec.constraints, constraints,
                "Constraints not preserved for case: {description}"
            );
        }
    }
}
