use crate::{
    contracts::SessionLib::TransferSpec as SessionLibTransferSpec,
    utils::session::session_lib::session_spec::usage_limit::UsageLimit,
};
use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferSpec {
    pub target: Address,
    pub max_value_per_use: U256,
    pub value_limit: UsageLimit,
}

impl From<SessionLibTransferSpec> for TransferSpec {
    fn from(value: SessionLibTransferSpec) -> Self {
        TransferSpec {
            target: value.target,
            max_value_per_use: value.maxValuePerUse,
            value_limit: value.valueLimit.into(),
        }
    }
}

impl From<TransferSpec> for SessionLibTransferSpec {
    fn from(value: TransferSpec) -> Self {
        SessionLibTransferSpec {
            target: value.target,
            maxValuePerUse: value.max_value_per_use,
            valueLimit: value.value_limit.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        contracts::SessionLib::UsageLimit as SessionLibUsageLimit,
        utils::session::session_lib::session_spec::limit_type::LimitType,
    };
    use alloy::primitives::{Address, U256, address};

    #[test]
    fn test_transfer_spec_round_trip_conversions() {
        let test_cases = vec![
            // Test case 1: Zero address with minimum values
            (
                Address::ZERO,
                U256::ZERO,
                UsageLimit {
                    limit_type: LimitType::Unlimited,
                    limit: U256::ZERO,
                    period: U256::ZERO,
                },
                "Zero address with unlimited usage limit",
            ),
            // Test case 2: Real address with lifetime limit
            (
                address!("0x1212121212121212121212121212121212121212"),
                U256::from(1000u64),
                UsageLimit {
                    limit_type: LimitType::Lifetime,
                    limit: U256::from(5000u64),
                    period: U256::from(3600u64),
                },
                "Real address with lifetime usage limit",
            ),
            // Test case 3: Different address with allowance limit
            (
                address!("0xABABABABABABABABABABABABABABABABABABABAB"),
                U256::from(500_000u64),
                UsageLimit {
                    limit_type: LimitType::Allowance,
                    limit: U256::from(1_000_000u64),
                    period: U256::from(86400u64),
                },
                "Different address with allowance usage limit",
            ),
            // Test case 4: Maximum values
            (
                address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),
                U256::MAX,
                UsageLimit {
                    limit_type: LimitType::Lifetime,
                    limit: U256::MAX,
                    period: U256::MAX,
                },
                "Maximum values with lifetime limit",
            ),
            // Test case 5: Mixed edge case
            (
                address!("0x112233445566778899AABBCCDDEEFF0012345678"),
                U256::from(12345u64),
                UsageLimit {
                    limit_type: LimitType::Allowance,
                    limit: U256::from(67890u64),
                    period: U256::ZERO,
                },
                "Mixed address with allowance and zero period",
            ),
        ];

        for (target, max_value_per_use, value_limit, description) in test_cases
        {
            // Create original SessionLibTransferSpec
            let original_session_lib = SessionLibTransferSpec {
                target,
                maxValuePerUse: max_value_per_use,
                valueLimit: SessionLibUsageLimit {
                    limitType: value_limit.limit_type.clone().into(),
                    limit: value_limit.limit,
                    period: value_limit.period,
                },
            };

            // Convert SessionLibTransferSpec -> TransferSpec
            let transfer_spec: TransferSpec =
                original_session_lib.clone().into();

            // Verify the conversion preserved all values
            assert_eq!(
                transfer_spec.target, target,
                "Target address mismatch for test case: {description}"
            );
            assert_eq!(
                transfer_spec.max_value_per_use, max_value_per_use,
                "Max value per use mismatch for test case: {description}"
            );
            assert_eq!(
                transfer_spec.value_limit, value_limit,
                "Value limit mismatch for test case: {description}"
            );

            // Convert TransferSpec -> SessionLibTransferSpec (round-trip)
            let round_trip_session_lib: SessionLibTransferSpec =
                transfer_spec.clone().into();

            // Verify the round-trip preserved all values
            assert_eq!(
                round_trip_session_lib.target, original_session_lib.target,
                "Round-trip target address mismatch for test case: {description}"
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

            // Additional verification: convert back to TransferSpec again
            let double_round_trip: TransferSpec = round_trip_session_lib.into();

            assert_eq!(
                double_round_trip.target, transfer_spec.target,
                "Double round-trip target mismatch for test case: {description}"
            );
            assert_eq!(
                double_round_trip.max_value_per_use,
                transfer_spec.max_value_per_use,
                "Double round-trip max value per use mismatch for test case: {description}"
            );
            assert_eq!(
                double_round_trip.value_limit, transfer_spec.value_limit,
                "Double round-trip value limit mismatch for test case: {description}"
            );
        }
    }

    #[test]
    fn test_transfer_spec_field_preservation() {
        // Test that each field is independently preserved during conversion
        let original = TransferSpec {
            target: address!("0x4242424242424242424242424242424242424242"),
            max_value_per_use: U256::from(999u64),
            value_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(1234u64),
                period: U256::from(5678u64),
            },
        };

        // Convert to SessionLibTransferSpec
        let session_lib: SessionLibTransferSpec = original.clone().into();

        // Verify each field individually
        assert_eq!(session_lib.target, original.target);
        assert_eq!(session_lib.maxValuePerUse, original.max_value_per_use);

        // Verify nested UsageLimit fields
        assert_eq!(session_lib.valueLimit.limit, original.value_limit.limit);
        assert_eq!(session_lib.valueLimit.period, original.value_limit.period);

        // Verify limit type numeric value
        let limit_type_numeric: u8 = session_lib.valueLimit.limitType;
        let expected_numeric: u8 =
            original.value_limit.limit_type.clone().into();
        assert_eq!(limit_type_numeric, expected_numeric);

        // Convert back and verify complete equality
        let round_trip: TransferSpec = session_lib.into();
        assert_eq!(round_trip, original);
    }

    #[test]
    fn test_transfer_spec_with_all_limit_types() {
        let base_target =
            address!("0x3333333333333333333333333333333333333333");
        let base_max_value = U256::from(777u64);
        let base_limit = U256::from(888u64);
        let base_period = U256::from(999u64);

        let limit_types = vec![
            (LimitType::Unlimited, 0u8),
            (LimitType::Lifetime, 1u8),
            (LimitType::Allowance, 2u8),
        ];

        for (limit_type, expected_numeric) in limit_types {
            let transfer_spec = TransferSpec {
                target: base_target,
                max_value_per_use: base_max_value,
                value_limit: UsageLimit {
                    limit_type: limit_type.clone(),
                    limit: base_limit,
                    period: base_period,
                },
            };

            // Convert to SessionLibTransferSpec
            let session_lib: SessionLibTransferSpec =
                transfer_spec.clone().into();

            // Verify limit type numeric value
            let numeric_value: u8 = session_lib.valueLimit.limitType;
            assert_eq!(
                numeric_value, expected_numeric,
                "Numeric value mismatch for limit type: {limit_type:?}"
            );

            // Convert back and verify equality
            let back_to_transfer_spec: TransferSpec = session_lib.into();
            assert_eq!(back_to_transfer_spec, transfer_spec);
        }
    }

    #[test]
    fn test_transfer_spec_equality_after_conversions() {
        let original = TransferSpec {
            target: address!("0xDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEF"),
            max_value_per_use: U256::from(12345u64),
            value_limit: UsageLimit {
                limit_type: LimitType::Allowance,
                limit: U256::from(67890u64),
                period: U256::from(11111u64),
            },
        };

        // Multiple conversion paths should all result in equivalent objects
        let path1 = {
            let session_lib: SessionLibTransferSpec = original.clone().into();
            let back: TransferSpec = session_lib.into();
            back
        };

        let path2 = {
            let session_lib: SessionLibTransferSpec = original.clone().into();
            let intermediate: TransferSpec = session_lib.into();
            let session_lib2: SessionLibTransferSpec = intermediate.into();
            let back: TransferSpec = session_lib2.into();
            back
        };

        assert_eq!(original, path1);
        assert_eq!(original, path2);
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_transfer_spec_address_variations() {
        // Test various address patterns
        let address_patterns = vec![
            (Address::ZERO, "Zero address"),
            (
                address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),
                "Max address",
            ),
            (
                address!("0x112233445566778899AABBCCDDEEFF0011223344"),
                "Pattern address",
            ),
            (
                address!("0x0100000000000000000000000000000000000001"),
                "Minimal non-zero address",
            ),
        ];

        for (address, description) in address_patterns {
            let transfer_spec = TransferSpec {
                target: address,
                max_value_per_use: U256::from(100u64),
                value_limit: UsageLimit::UNLIMITED,
            };

            // Test round-trip conversion
            let session_lib: SessionLibTransferSpec =
                transfer_spec.clone().into();
            let back_to_transfer_spec: TransferSpec = session_lib.into();

            assert_eq!(
                back_to_transfer_spec, transfer_spec,
                "Round-trip failed for address case: {description}"
            );
            assert_eq!(
                back_to_transfer_spec.target, address,
                "Address not preserved for case: {description}"
            );
        }
    }
}
