use crate::erc4337::account::modular_smart_account::session::SessionLib::Condition as SessionLibCondition;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum Condition {
    Unconstrained,
    Equal,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    NotEqual,
}

impl From<Condition> for u8 {
    fn from(value: Condition) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for Condition {
    type Error = eyre::Report;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == Condition::Unconstrained as u8 => {
                Ok(Condition::Unconstrained)
            }
            x if x == Condition::Equal as u8 => Ok(Condition::Equal),
            x if x == Condition::Greater as u8 => Ok(Condition::Greater),
            x if x == Condition::Less as u8 => Ok(Condition::Less),
            x if x == Condition::GreaterEqual as u8 => {
                Ok(Condition::GreaterEqual)
            }
            x if x == Condition::LessEqual as u8 => Ok(Condition::LessEqual),
            x if x == Condition::NotEqual as u8 => Ok(Condition::NotEqual),
            _ => Err(eyre::eyre!("Invalid condition value: {}", value)),
        }
    }
}

impl TryFrom<SessionLibCondition> for Condition {
    type Error = eyre::Report;

    fn try_from(value: SessionLibCondition) -> Result<Self, Self::Error> {
        let value: u8 = value.into();
        Condition::try_from(value)
    }
}

impl From<Condition> for SessionLibCondition {
    fn from(val: Condition) -> Self {
        let value: u8 = val.into();
        SessionLibCondition::from(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_numeric_values() {
        // Test that each enum variant maps to the expected numeric value
        assert_eq!(Condition::Unconstrained as u8, 0);
        assert_eq!(Condition::Equal as u8, 1);
        assert_eq!(Condition::Greater as u8, 2);
        assert_eq!(Condition::Less as u8, 3);
        assert_eq!(Condition::GreaterEqual as u8, 4);
        assert_eq!(Condition::LessEqual as u8, 5);
        assert_eq!(Condition::NotEqual as u8, 6);
    }

    #[test]
    fn test_condition_u8_conversions() {
        let test_cases = vec![
            (Condition::Unconstrained, 0u8),
            (Condition::Equal, 1u8),
            (Condition::Greater, 2u8),
            (Condition::Less, 3u8),
            (Condition::GreaterEqual, 4u8),
            (Condition::LessEqual, 5u8),
            (Condition::NotEqual, 6u8),
        ];

        for (condition, expected_value) in test_cases {
            // Test Condition -> u8
            let u8_value: u8 = condition.clone().into();
            assert_eq!(u8_value, expected_value);

            // Test u8 -> Condition
            let round_trip_condition =
                Condition::try_from(expected_value).unwrap();
            assert_eq!(round_trip_condition, condition);
        }
    }

    #[test]
    fn test_condition_round_trip_conversions() {
        let conditions = vec![
            Condition::Unconstrained,
            Condition::Equal,
            Condition::Greater,
            Condition::Less,
            Condition::GreaterEqual,
            Condition::LessEqual,
            Condition::NotEqual,
        ];

        for original_condition in conditions {
            // Test Condition -> u8 -> Condition
            let u8_value: u8 = original_condition.clone().into();
            let round_trip_condition = Condition::try_from(u8_value).unwrap();
            assert_eq!(round_trip_condition, original_condition);
        }
    }

    #[test]
    fn test_session_lib_condition_conversions() -> eyre::Result<()> {
        let test_cases = vec![
            (0u8, Condition::Unconstrained),
            (1u8, Condition::Equal),
            (2u8, Condition::Greater),
            (3u8, Condition::Less),
            (4u8, Condition::GreaterEqual),
            (5u8, Condition::LessEqual),
            (6u8, Condition::NotEqual),
        ];

        for (raw_value, expected_condition) in test_cases {
            // Test the conversion path: u8 -> SessionLibCondition -> Condition
            let session_lib_condition = SessionLibCondition::from(raw_value);
            let condition: Condition = session_lib_condition.try_into()?;

            // Verify the condition is the expected value
            assert_eq!(condition, expected_condition);

            // Verify round-trip: Condition -> u8 -> Condition
            let u8_value: u8 = condition.clone().into();
            assert_eq!(u8_value, raw_value);

            let round_trip_condition = Condition::try_from(u8_value)?;
            assert_eq!(round_trip_condition, condition);

            // Test Condition -> SessionLibCondition round-trip
            let back_to_session_lib: SessionLibCondition =
                condition.clone().into();
            let back_to_condition: Condition =
                back_to_session_lib.try_into()?;
            assert_eq!(back_to_condition, condition);
        }

        Ok(())
    }

    #[test]
    fn test_all_conversions_equivalent() -> eyre::Result<()> {
        // Test that all conversion paths lead to equivalent results
        let raw_values = [0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8];

        for raw_value in raw_values {
            // Path 1: raw_value -> Condition directly
            let condition_path1 = Condition::try_from(raw_value)?;

            // Path 2: raw_value -> SessionLibCondition -> Condition
            let session_lib_condition = SessionLibCondition::from(raw_value);
            let condition_path2: Condition =
                session_lib_condition.try_into()?;

            // Both paths should produce equivalent results
            assert_eq!(condition_path1, condition_path2);

            // Both should convert back to the same raw value
            let back_to_u8_path1: u8 = condition_path1.clone().into();
            let back_to_u8_path2: u8 = condition_path2.clone().into();

            assert_eq!(back_to_u8_path1, raw_value);
            assert_eq!(back_to_u8_path2, raw_value);
        }

        Ok(())
    }

    #[test]
    fn test_invalid_u8_conversion() {
        // Test that invalid u8 values return errors
        let invalid_values = [7u8, 8u8, 255u8];

        for invalid_value in invalid_values {
            assert!(Condition::try_from(invalid_value).is_err());
        }
    }
}
