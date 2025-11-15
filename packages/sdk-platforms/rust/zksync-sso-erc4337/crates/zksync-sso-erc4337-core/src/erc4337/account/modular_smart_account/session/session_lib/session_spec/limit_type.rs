use crate::erc4337::account::modular_smart_account::session::contract::SessionLib::LimitType as SessionLibLimitType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum LimitType {
    Unlimited,
    Lifetime,
    Allowance,
}

impl From<LimitType> for u8 {
    fn from(value: LimitType) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for LimitType {
    type Error = eyre::Report;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == LimitType::Unlimited as u8 => Ok(LimitType::Unlimited),
            x if x == LimitType::Lifetime as u8 => Ok(LimitType::Lifetime),
            x if x == LimitType::Allowance as u8 => Ok(LimitType::Allowance),
            _ => Err(eyre::eyre!("Invalid limit type value: {}", value)),
        }
    }
}

impl TryFrom<SessionLibLimitType> for LimitType {
    type Error = eyre::Report;

    fn try_from(value: SessionLibLimitType) -> Result<Self, Self::Error> {
        let value: u8 = value.into();
        LimitType::try_from(value)
    }
}

impl From<LimitType> for SessionLibLimitType {
    fn from(val: LimitType) -> Self {
        let value: u8 = val.into();
        SessionLibLimitType::from(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limit_type_numeric_values() {
        // Test that each enum variant maps to the expected numeric value
        assert_eq!(LimitType::Unlimited as u8, 0);
        assert_eq!(LimitType::Lifetime as u8, 1);
        assert_eq!(LimitType::Allowance as u8, 2);
    }

    #[test]
    fn test_limit_type_u8_conversions() {
        let test_cases = vec![
            (LimitType::Unlimited, 0u8),
            (LimitType::Lifetime, 1u8),
            (LimitType::Allowance, 2u8),
        ];

        for (limit_type, expected_value) in test_cases {
            // Test LimitType -> u8
            let u8_value: u8 = limit_type.clone().into();
            assert_eq!(u8_value, expected_value);

            // Test u8 -> LimitType
            let round_trip_limit_type =
                LimitType::try_from(expected_value).unwrap();
            assert_eq!(round_trip_limit_type, limit_type);
        }
    }

    #[test]
    fn test_limit_type_round_trip_conversions() {
        let limit_types = vec![
            LimitType::Unlimited,
            LimitType::Lifetime,
            LimitType::Allowance,
        ];

        for original_limit_type in limit_types {
            // Test LimitType -> u8 -> LimitType
            let u8_value: u8 = original_limit_type.clone().into();
            let round_trip_limit_type = LimitType::try_from(u8_value).unwrap();
            assert_eq!(round_trip_limit_type, original_limit_type);
        }
    }

    #[test]
    fn test_session_lib_limit_type_conversions() -> eyre::Result<()> {
        let test_cases = vec![
            (0u8, LimitType::Unlimited),
            (1u8, LimitType::Lifetime),
            (2u8, LimitType::Allowance),
        ];

        for (raw_value, expected_limit_type) in test_cases {
            // Test the conversion path: u8 -> SessionLibLimitType -> LimitType
            let session_lib_limit_type = SessionLibLimitType::from(raw_value);
            let limit_type: LimitType =
                LimitType::try_from(session_lib_limit_type)?;

            // Verify the limit type is the expected value
            assert_eq!(limit_type, expected_limit_type);

            // Verify round-trip: LimitType -> u8 -> LimitType
            let u8_value: u8 = limit_type.clone().into();
            assert_eq!(u8_value, raw_value);

            let round_trip_limit_type = LimitType::try_from(u8_value)?;
            assert_eq!(round_trip_limit_type, limit_type);

            // Test LimitType -> SessionLibLimitType round-trip
            let back_to_session_lib: SessionLibLimitType =
                limit_type.clone().into();
            let back_to_limit_type: LimitType =
                LimitType::try_from(back_to_session_lib)?;
            assert_eq!(back_to_limit_type, limit_type);
        }

        Ok(())
    }

    #[test]
    fn test_all_conversions_equivalent() -> eyre::Result<()> {
        // Test that all conversion paths lead to equivalent results
        let raw_values = [0u8, 1u8, 2u8];

        for raw_value in raw_values {
            // Path 1: raw_value -> LimitType directly
            let limit_type_path1 = LimitType::try_from(raw_value)?;

            // Path 2: raw_value -> SessionLibLimitType -> LimitType
            let session_lib_limit_type = SessionLibLimitType::from(raw_value);
            let limit_type_path2: LimitType =
                LimitType::try_from(session_lib_limit_type)?;

            // Both paths should produce equivalent results
            assert_eq!(limit_type_path1, limit_type_path2);

            // Both should convert back to the same raw value
            let back_to_u8_path1: u8 = limit_type_path1.clone().into();
            let back_to_u8_path2: u8 = limit_type_path2.clone().into();

            assert_eq!(back_to_u8_path1, raw_value);
            assert_eq!(back_to_u8_path2, raw_value);
        }

        Ok(())
    }

    #[test]
    fn test_invalid_u8_conversion() {
        // Test that invalid u8 values return errors
        let invalid_values = [3u8, 4u8, 255u8];

        for invalid_value in invalid_values {
            assert!(LimitType::try_from(invalid_value).is_err());
        }
    }
}
