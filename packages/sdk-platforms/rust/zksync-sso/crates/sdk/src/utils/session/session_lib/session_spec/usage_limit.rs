use crate::{
    contracts::SessionLib::UsageLimit as SessionLibUsageLimit,
    utils::session::session_lib::session_spec::limit_type::LimitType,
};
use alloy::primitives::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UsageLimit {
    pub limit_type: LimitType,
    pub limit: U256,
    pub period: U256,
}

impl UsageLimit {
    pub const UNLIMITED: UsageLimit = UsageLimit {
        limit_type: LimitType::Unlimited,
        limit: U256::ZERO,
        period: U256::ZERO,
    };

    pub const ZERO: UsageLimit = UsageLimit {
        limit_type: LimitType::Lifetime,
        limit: U256::ZERO,
        period: U256::ZERO,
    };
}

impl From<SessionLibUsageLimit> for UsageLimit {
    fn from(value: SessionLibUsageLimit) -> Self {
        UsageLimit {
            limit_type: value.limitType.try_into().unwrap(),
            limit: value.limit,
            period: value.period,
        }
    }
}

impl From<UsageLimit> for SessionLibUsageLimit {
    fn from(value: UsageLimit) -> Self {
        SessionLibUsageLimit {
            limitType: value.limit_type.into(),
            limit: value.limit,
            period: value.period,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::U256;

    #[test]
    fn test_usage_limit_constants() {
        // Test UNLIMITED constant
        assert_eq!(UsageLimit::UNLIMITED.limit_type, LimitType::Unlimited);
        assert_eq!(UsageLimit::UNLIMITED.limit, U256::ZERO);
        assert_eq!(UsageLimit::UNLIMITED.period, U256::ZERO);

        // Test ZERO constant
        assert_eq!(UsageLimit::ZERO.limit_type, LimitType::Lifetime);
        assert_eq!(UsageLimit::ZERO.limit, U256::ZERO);
        assert_eq!(UsageLimit::ZERO.period, U256::ZERO);
    }

    #[test]
    fn test_session_lib_usage_limit_round_trip_conversions() {
        let test_cases = vec![
            // Test case 1: Unlimited type
            (
                LimitType::Unlimited,
                U256::from(0u64),
                U256::from(0u64),
                "Unlimited with zero values",
            ),
            // Test case 2: Lifetime type with specific values
            (
                LimitType::Lifetime,
                U256::from(1000u64),
                U256::from(3600u64),
                "Lifetime with 1000 limit and 3600 period",
            ),
            // Test case 3: Allowance type with large values
            (
                LimitType::Allowance,
                U256::from(1_000_000u64),
                U256::from(86400u64),
                "Allowance with 1M limit and 86400 period",
            ),
            // Test case 4: Maximum U256 values
            (
                LimitType::Lifetime,
                U256::MAX,
                U256::MAX,
                "Lifetime with maximum U256 values",
            ),
            // Test case 5: Edge case with one field as zero
            (
                LimitType::Allowance,
                U256::from(500u64),
                U256::ZERO,
                "Allowance with non-zero limit and zero period",
            ),
        ];

        for (limit_type, limit, period, description) in test_cases {
            // Create original SessionLibUsageLimit
            let original_session_lib = SessionLibUsageLimit {
                limitType: limit_type.clone().into(),
                limit,
                period,
            };

            // Convert SessionLibUsageLimit -> UsageLimit
            let usage_limit: UsageLimit = original_session_lib.clone().into();

            // Verify the conversion preserved all values
            assert_eq!(
                usage_limit.limit_type, limit_type,
                "Limit type mismatch for test case: {description}"
            );
            assert_eq!(
                usage_limit.limit, limit,
                "Limit value mismatch for test case: {description}"
            );
            assert_eq!(
                usage_limit.period, period,
                "Period value mismatch for test case: {description}"
            );

            // Convert UsageLimit -> SessionLibUsageLimit (round-trip)
            let round_trip_session_lib: SessionLibUsageLimit =
                usage_limit.clone().into();

            // Verify the round-trip preserved all values
            assert_eq!(
                round_trip_session_lib.limitType,
                original_session_lib.limitType,
                "Round-trip limit type mismatch for test case: {description}"
            );
            assert_eq!(
                round_trip_session_lib.limit, original_session_lib.limit,
                "Round-trip limit value mismatch for test case: {description}"
            );
            assert_eq!(
                round_trip_session_lib.period, original_session_lib.period,
                "Round-trip period value mismatch for test case: {description}"
            );

            // Additional verification: convert back to UsageLimit again
            let double_round_trip: UsageLimit = round_trip_session_lib.into();

            assert_eq!(
                double_round_trip.limit_type, usage_limit.limit_type,
                "Double round-trip limit type mismatch for test case: {description}"
            );
            assert_eq!(
                double_round_trip.limit, usage_limit.limit,
                "Double round-trip limit value mismatch for test case: {description}"
            );
            assert_eq!(
                double_round_trip.period, usage_limit.period,
                "Double round-trip period value mismatch for test case: {description}"
            );
        }
    }

    #[test]
    fn test_usage_limit_from_constants() {
        // Test converting constants to SessionLibUsageLimit and back
        let unlimited_session_lib: SessionLibUsageLimit =
            UsageLimit::UNLIMITED.into();
        let unlimited_back: UsageLimit = unlimited_session_lib.into();

        assert_eq!(unlimited_back, UsageLimit::UNLIMITED);

        let zero_session_lib: SessionLibUsageLimit = UsageLimit::ZERO.into();
        let zero_back: UsageLimit = zero_session_lib.into();

        assert_eq!(zero_back, UsageLimit::ZERO);
    }

    #[test]
    fn test_usage_limit_specific_limit_type_values() {
        // Test that the underlying numeric values are preserved correctly
        let test_cases = vec![
            (LimitType::Unlimited, 0u8),
            (LimitType::Lifetime, 1u8),
            (LimitType::Allowance, 2u8),
        ];

        for (limit_type, expected_numeric) in test_cases {
            let usage_limit = UsageLimit {
                limit_type: limit_type.clone(),
                limit: U256::from(100u64),
                period: U256::from(200u64),
            };

            // Convert to SessionLibUsageLimit
            let session_lib: SessionLibUsageLimit = usage_limit.into();

            // Verify the numeric value matches expected
            let numeric_value: u8 = session_lib.limitType;
            assert_eq!(
                numeric_value, expected_numeric,
                "Numeric value mismatch for limit type: {limit_type:?}"
            );

            // Convert back and verify
            let back_to_usage_limit: UsageLimit = session_lib.into();
            assert_eq!(back_to_usage_limit.limit_type, limit_type);
        }
    }

    #[test]
    fn test_usage_limit_equality_after_conversions() {
        let original = UsageLimit {
            limit_type: LimitType::Allowance,
            limit: U256::from(12345u64),
            period: U256::from(67890u64),
        };

        // Multiple conversion paths should all result in equivalent objects
        let path1 = {
            let session_lib: SessionLibUsageLimit = original.clone().into();
            let back: UsageLimit = session_lib.into();
            back
        };

        let path2 = {
            let session_lib: SessionLibUsageLimit = original.clone().into();
            let intermediate: UsageLimit = session_lib.into();
            let session_lib2: SessionLibUsageLimit = intermediate.into();
            let back: UsageLimit = session_lib2.into();
            back
        };

        assert_eq!(original, path1);
        assert_eq!(original, path2);
        assert_eq!(path1, path2);
    }
}
