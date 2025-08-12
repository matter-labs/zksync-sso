use crate::{
    contracts::SessionLib::SessionSpec as SessionLibSessionSpec,
    utils::session::session_lib::session_spec::SessionSpec,
};
use alloy::{primitives::Bytes, sol, sol_types::SolType};
use eyre::Result;
use log::debug;

pub mod transaction;

sol! {
    struct SessionParams {
        SessionLibSessionSpec memory sessionSpec;
    }
}

pub fn encode_session_key_module_parameters(
    session_spec: SessionSpec,
) -> Result<Bytes> {
    debug!(
        "XDB encode_passkey_module_parameters - session_spec: {session_spec:?}"
    );

    let params = SessionParams { sessionSpec: session_spec.into() };

    let params_bytes = <SessionParams as SolType>::abi_encode_params(&params);

    Ok(params_bytes.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::session::session_lib::session_spec::{
        SessionSpec, call_spec::CallSpec, condition::Condition,
        constraint::Constraint, limit_type::LimitType,
        transfer_spec::TransferSpec, usage_limit::UsageLimit,
    };
    use alloy::primitives::{FixedBytes, U256, address, hex};

    #[test]
    fn test_encode_session_key_module_parameters() -> eyre::Result<()> {
        // Arrange - using values from test_deploy_account_with_initial_session
        let signer_address =
            address!("0x9BbC92a33F193174bf6Cc09c4b4055500d972479");

        let session_spec = SessionSpec {
            signer: signer_address,
            expires_at: U256::from(1749040108u64), // Future timestamp
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime, // Lifetime limit
                limit: U256::from(100000000000000000u64), // 0.1 ETH
                period: U256::from(0),
            },
            call_policies: vec![], // No call policies for this test
            transfer_policies: vec![TransferSpec {
                target: address!("0xdeBbD4CE2Bd6BD869D3ac93666A0D5F4fc06FC72"),
                max_value_per_use: U256::from(10000000000000000u64), // 0.01 ETH
                value_limit: UsageLimit {
                    limit_type: LimitType::Unlimited, // Unlimited
                    limit: U256::from(0),
                    period: U256::from(0),
                },
            }],
        };

        // Act
        let result =
            encode_session_key_module_parameters(session_spec.clone())?;

        // Assert
        let hex_result = format!("0x{}", hex::encode(&result));

        // Verify the result is not empty
        eyre::ensure!(!result.is_empty(), "Encoded result should not be empty");

        // Verify the result contains the signer address
        eyre::ensure!(
            hex_result.contains(&hex::encode(signer_address.as_slice())),
            "Encoded result should contain signer address"
        );

        // Verify the result contains the expires_at timestamp
        let expires_at_bytes: [u8; 32] = session_spec.expires_at.to_be_bytes();
        eyre::ensure!(
            hex_result.contains(&hex::encode(&expires_at_bytes[4..])), // Skip leading zeros
            "Encoded result should contain expiresAt timestamp"
        );

        // Verify the result contains the transfer policy target
        eyre::ensure!(
            hex_result.contains(&hex::encode(
                session_spec.transfer_policies[0].target.as_slice()
            )),
            "Encoded result should contain transfer policy target address"
        );

        // Verify the result contains the fee limit amount
        let fee_limit_bytes: [u8; 32] =
            session_spec.fee_limit.limit.to_be_bytes();
        eyre::ensure!(
            hex_result.contains(&hex::encode(&fee_limit_bytes[1..])), // Skip leading zero
            "Encoded result should contain fee limit amount"
        );

        println!("Encoded session parameters: {hex_result}");
        println!("Result length: {} bytes", result.len());

        Ok(())
    }

    #[test]
    fn test_encode_session_key_module_parameters_js() -> eyre::Result<()> {
        // Arrange - using the exact same complex session configuration as the JavaScript test
        let signer_address =
            address!("0x9BbC92a33F193174bf6Cc09c4b4055500d972479");

        // Build the complex session spec that matches the JavaScript test exactly
        let session_spec = SessionSpec {
            signer: signer_address,
            expires_at: U256::from(1749040108u64), // Static timestamp for deterministic testing
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime, // LimitType.Lifetime = 1
                limit: U256::from(100000000000000000u64), // parseEther("0.1")
                period: U256::from(0),
            },
            call_policies: vec![CallSpec {
                target: address!("0x1111111111111111111111111111111111111111"),
                selector: FixedBytes::from([0xa9, 0x05, 0x9c, 0xbb]), // "0xa9059cbb" - transfer(address,uint256)
                max_value_per_use: U256::from(1000000000000000u64), // parseEther("0.001")
                value_limit: UsageLimit {
                    limit_type: LimitType::Allowance, // LimitType.Allowance = 2
                    limit: U256::from(10000000000000000u64), // parseEther("0.01")
                    period: U256::from(3600u64),             // 1 hour
                },
                constraints: vec![Constraint {
                    condition: Condition::Equal, // Condition.Equal = 1
                    index: 4, // First parameter after selector (recipient address)
                    ref_value: FixedBytes::from([0u8; 32]), // ethers.ZeroHash
                    limit: UsageLimit {
                        limit_type: LimitType::Allowance, // LimitType.Allowance = 2
                        limit: U256::from(5000000000000000u64), // parseEther("0.005")
                        period: U256::from(1800u64),            // 30 minutes
                    },
                }],
            }],
            transfer_policies: vec![TransferSpec {
                target: address!("0x0000000000000000000000000000000000000000"), // ethers.ZeroAddress
                max_value_per_use: U256::from(20000000000000000u64), // parseEther("0.02")
                value_limit: UsageLimit {
                    limit_type: LimitType::Allowance, // LimitType.Allowance = 2
                    limit: U256::from(50000000000000000u64), // parseEther("0.05")
                    period: U256::from(86400u64),            // 24 hours
                },
            }],
        };

        // Act
        let result =
            encode_session_key_module_parameters(session_spec.clone())?;

        // Assert
        let hex_result = format!("0x{}", hex::encode(&result));

        // Expected result from the JavaScript test
        let expected_hex = "0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000009bbc92a33f193174bf6cc09c4b4055500d9724790000000000000000000000000000000000000000000000000000000068403bec0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000002e0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000001111111111111111111111111111111111111111a9059cbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000038d7ea4c680000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000002386f26fc100000000000000000000000000000000000000000000000000000000000000000e1000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000011c37937e0800000000000000000000000000000000000000000000000000000000000000007080000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000470de4df820000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000b1a2bc2ec500000000000000000000000000000000000000000000000000000000000000015180";

        println!("=== JS COMPLEX SESSION ENCODING TEST ===");
        println!("Rust encoded result: {hex_result}");
        println!("JS expected result:  {expected_hex}");
        println!("Result length: {} bytes", result.len());
        println!("Expected length: {} bytes", expected_hex.len() / 2 - 1); // -1 for 0x prefix

        // Verify the encoding matches the JavaScript result exactly
        eyre::ensure!(
            hex_result == expected_hex,
            "Rust encoding should match JavaScript encoding exactly.\nRust:     {}\nExpected: {}",
            hex_result,
            expected_hex
        );

        // Additional verification - check that all components are present
        // 1. Signer address
        eyre::ensure!(
            hex_result.contains(&hex::encode(signer_address.as_slice())),
            "Encoded result should contain signer address"
        );

        // 2. Expires at timestamp
        let expires_at_bytes: [u8; 32] = session_spec.expires_at.to_be_bytes();
        eyre::ensure!(
            hex_result.contains(&hex::encode(&expires_at_bytes[4..])), // Skip leading zeros
            "Encoded result should contain expiresAt timestamp"
        );

        // 3. Call policy target
        eyre::ensure!(
            hex_result.contains(&hex::encode(
                session_spec.call_policies[0].target.as_slice()
            )),
            "Encoded result should contain call policy target address"
        );

        // 4. Call policy selector
        eyre::ensure!(
            hex_result.contains(&hex::encode(
                session_spec.call_policies[0].selector.as_slice()
            )),
            "Encoded result should contain call policy selector"
        );

        // 5. Transfer policy target (zero address)
        eyre::ensure!(
            hex_result.contains(&hex::encode(
                session_spec.transfer_policies[0].target.as_slice()
            )),
            "Encoded result should contain transfer policy target address"
        );

        println!(
            "✅ All verification checks passed - Rust encoding matches JavaScript exactly!"
        );

        Ok(())
    }
}
