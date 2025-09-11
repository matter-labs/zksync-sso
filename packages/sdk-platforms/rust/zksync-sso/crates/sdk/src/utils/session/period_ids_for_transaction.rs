use crate::{
    contracts::SessionLib::{CallSpec, SessionSpec, TransferSpec, UsageLimit},
    utils::session::session_lib::session_spec::{
        Policy, limit_type::LimitType,
    },
};
use alloy::primitives::{Address, FixedBytes, U64};
use log::debug;

/// Gets the period IDs for a transaction based on the session config and transaction details
///
/// # Arguments
/// * `session_config` - The session configuration containing policies
/// * `target` - The target address for the transaction
/// * `selector` - Optional function selector for contract calls
/// * `timestamp` - Optional timestamp to use (defaults to current time)
///
/// # Returns
/// A vector of period IDs for the transaction
///
/// # Errors
/// Returns an error if the transaction doesn't fit any policy
pub fn get_period_ids_for_transaction(
    session_config: &SessionSpec,
    target: Address,
    selector: Option<FixedBytes<4>>,
    timestamp: Option<U64>,
) -> eyre::Result<Vec<U64>> {
    let timestamp = timestamp.unwrap_or_else(|| {
        U64::from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
    });

    let get_id = |usage_limit: &UsageLimit| -> U64 {
        let limit_type: LimitType = usage_limit.limitType.try_into().unwrap();
        match limit_type {
            LimitType::Allowance => {
                let period_u64 =
                    usage_limit.period.try_into().unwrap_or(u64::MAX);
                timestamp / U64::from(period_u64)
            }
            _ => U64::ZERO,
        }
    };

    println!("selector: {selector:?}");
    let is_contract_call = selector.is_some();

    println!("is_contract_call: {is_contract_call:?}");

    let find_transfer_policy =
        |transfer_policies: Vec<TransferSpec>| -> Option<TransferSpec> {
            debug!(
                "find_transfer_policy - transfer_policies: {transfer_policies:?}"
            );
            debug!("find_transfer_policy - target: {target:?}");
            transfer_policies
                .iter()
                .find(|policy| policy.target == target)
                .map(ToOwned::to_owned)
        };

    let find_call_policy = |call_policies: Vec<CallSpec>| -> Option<CallSpec> {
        call_policies
            .iter()
            .find(|policy| {
                policy.target == target && (selector == Some(policy.selector))
            })
            .map(ToOwned::to_owned)
    };

    let policy = {
        let policy = if is_contract_call {
            find_call_policy(session_config.callPolicies.clone())
                .map(Policy::from)
        } else {
            find_transfer_policy(session_config.transferPolicies.clone())
                .map(Policy::from)
        };
        policy.ok_or(eyre::eyre!("Transaction does not fit any policy"))?
    };

    let period_ids = {
        let usage_limit = policy.clone().usage_limit().to_owned().into();
        let mut period_ids =
            vec![get_id(&session_config.feeLimit), get_id(&usage_limit)];
        if is_contract_call && let Some(call_policy) = policy.as_call_policy() {
            period_ids.extend(
                call_policy
                    .constraints
                    .iter()
                    .map(|constraint| get_id(&constraint.clone().limit.into())),
            );
        }
        period_ids
    };

    Ok(period_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::SessionLib::{Constraint, TransferSpec};
    use alloy::primitives::{FixedBytes, U256, address, hex};

    // Helper function to create mock session config matching the TypeScript version
    fn create_mock_session_config() -> SessionSpec {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        SessionSpec {
            signer: address!("1234567890123456789012345678901234567890"),
            expiresAt: U256::from(now + 86400), // expires in 1 day
            feeLimit: UsageLimit {
                limitType: LimitType::Allowance as u8, // Allowance
                limit: U256::from(100000000000000u64),
                period: U256::from(3600u64), // 1 hour
            },
            callPolicies: vec![CallSpec {
                target: address!("aAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa"),
                selector: FixedBytes::from(hex!("12345678")),
                maxValuePerUse: U256::from(1000000000000u64),
                valueLimit: UsageLimit {
                    limitType: LimitType::Allowance as u8, // Allowance
                    limit: U256::from(10000000000000u64),
                    period: U256::from(86400u64), // 1 day
                },
                constraints: vec![
                    Constraint {
                        condition: 1, // Equal
                        index: 4u64,
                        refValue: FixedBytes::from(hex!(
                            "0000000000000000000000000000000000000000000000000000000000000001"
                        )),
                        limit: UsageLimit {
                            limitType: LimitType::Lifetime as u8, // Lifetime
                            limit: U256::from(1000000000000u64),
                            period: U256::ZERO,
                        },
                    },
                    Constraint {
                        condition: 4, // GreaterEqual
                        index: 36u64,
                        refValue: FixedBytes::from(hex!(
                            "0000000000000000000000000000000000000000000000000000000000000000"
                        )),
                        limit: UsageLimit {
                            limitType: LimitType::Allowance as u8, // Allowance
                            limit: U256::from(5000000000000u64),
                            period: U256::from(43200u64), // 12 hours
                        },
                    },
                ],
            }],
            transferPolicies: vec![TransferSpec {
                target: address!("bBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"),
                maxValuePerUse: U256::from(500000000000u64),
                valueLimit: UsageLimit {
                    limitType: LimitType::Allowance as u8, // Allowance
                    limit: U256::from(5000000000000u64),
                    period: U256::from(604800u64), // 1 week
                },
            }],
        }
    }

    #[test]
    fn test_get_period_ids_for_transaction_contract_call() -> eyre::Result<()> {
        let mock_session_config = create_mock_session_config();
        let timestamp = U64::from(1714500000u64); // Fixed timestamp

        let period_ids = get_period_ids_for_transaction(
            &mock_session_config,
            address!("aAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa"),
            Some(FixedBytes::from(hex!("12345678"))),
            Some(timestamp),
        )?;

        // Should return period IDs for fee limit, value limit, and both constraints
        assert_eq!(period_ids.len(), 4);

        // Check fee limit period ID (timestamp / hourly period)
        assert_eq!(period_ids[0], timestamp / U64::from(3600u64));

        // Check value limit period ID (timestamp / daily period)
        assert_eq!(period_ids[1], timestamp / U64::from(86400u64));

        // Check first constraint (lifetime, so period ID is 0)
        assert_eq!(period_ids[2], U64::ZERO);

        // Check second constraint (12 hour period)
        assert_eq!(period_ids[3], timestamp / U64::from(43200u64));

        Ok(())
    }

    #[test]
    fn test_get_period_ids_for_transaction_transfer() -> eyre::Result<()> {
        let mock_session_config = create_mock_session_config();
        let timestamp = U64::from(1714500000u64); // Fixed timestamp

        let period_ids = get_period_ids_for_transaction(
            &mock_session_config,
            address!("bBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"),
            None,
            Some(timestamp),
        )?;

        // Should return period IDs for fee limit and value limit only (no constraints for transfers)
        assert_eq!(period_ids.len(), 2);

        // Check fee limit period ID (timestamp / hourly period)
        assert_eq!(period_ids[0], timestamp / U64::from(3600u64));

        // Check value limit period ID (timestamp / weekly period)
        assert_eq!(period_ids[1], timestamp / U64::from(604800u64));

        Ok(())
    }

    #[test]
    fn test_get_period_ids_for_transaction_current_timestamp()
    -> eyre::Result<()> {
        let mock_session_config = create_mock_session_config();

        let period_ids = get_period_ids_for_transaction(
            &mock_session_config,
            address!("bBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"),
            None,
            None, // Uses current timestamp
        )?;

        // Should still return 2 period IDs for transfer
        assert_eq!(period_ids.len(), 2);

        // Period IDs should be calculated using current timestamp
        let now = U64::from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        let expected_fee_id = now / U64::from(3600u64);
        let expected_value_id = now / U64::from(604800u64);

        // Allow a small margin for test execution time
        assert!(period_ids[0] >= expected_fee_id - U64::from(1u64));
        assert!(period_ids[0] <= expected_fee_id + U64::from(1u64));

        assert_eq!(period_ids[1], expected_value_id);

        Ok(())
    }

    #[test]
    fn test_get_period_ids_for_transaction_no_matching_policy() {
        let mock_session_config = create_mock_session_config();

        // Test with non-matching address
        let result = get_period_ids_for_transaction(
            &mock_session_config,
            address!("CcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"),
            None,
            Some(U64::from(1714500000u64)),
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Transaction does not fit any policy")
        );

        // Test with matching address but wrong selector
        let result = get_period_ids_for_transaction(
            &mock_session_config,
            address!("aAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa"),
            Some(FixedBytes::from(hex!("87654321"))), // Different selector
            Some(U64::from(1714500000u64)),
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Transaction does not fit any policy")
        );
    }
}
