use crate::{
    client::session::{
        actions::session::sign::sign_hash::{
            CreateTransactionSessionSignedHashParameters,
            create_transaction_session_signed_hash,
        },
        client::{
            SessionTransactionEncodedParamsArgs,
            encoded_session_transaction_signature,
        },
    },
    config::contracts::SSOContracts,
    utils::session::session_lib::session_spec::SessionSpec,
};
use alloy::primitives::{Address, Bytes, FixedBytes, U64};
use log::debug;

pub mod session_signer;
pub mod sign_hash;

#[derive(Clone, Debug)]
pub(crate) struct CreateSessionTransactionSignatureParameters {
    pub hash: FixedBytes<32>,
    pub to: Address,
    pub call_data: Option<Bytes>,
    pub session_key: FixedBytes<32>,
    pub session_config: SessionSpec,
    pub contracts: SSOContracts,
    pub timestamp: Option<U64>,
}

pub(crate) fn create_session_transaction_signature(
    parameters: CreateSessionTransactionSignatureParameters,
) -> eyre::Result<Bytes> {
    debug!("create_session_transaction_signature_alt");
    debug!("  parameters: {parameters:?}");

    let hash = parameters.hash;
    let to = parameters.to;
    let call_data = parameters.call_data;
    let timestamp = parameters.timestamp;
    let session_contract = parameters.contracts.session;
    let session_config = parameters.session_config;
    let session_key = parameters.session_key;

    let session_key_signed_hash = create_transaction_session_signed_hash(
        CreateTransactionSessionSignedHashParameters { hash, session_key },
    )?;

    let signature = encoded_session_transaction_signature(
        SessionTransactionEncodedParamsArgs {
            session_key_signed_hash,
            session_contract,
            session_config,
            to,
            call_data,
            timestamp,
        },
    )?;

    Ok(signature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::contracts::SSOContracts,
        utils::session::session_lib::session_spec::{
            SessionSpec, call_spec::CallSpec, condition::Condition,
            constraint::Constraint, limit_type::LimitType,
            transfer_spec::TransferSpec, usage_limit::UsageLimit,
        },
    };
    use alloy::primitives::{U64, U256, address, bytes, fixed_bytes, hex};

    #[test]
    #[ignore = "TODO: investigate why this test is failing - look at JS inputs/outputs to check"]
    fn test_create_session_transaction_signature_alt() -> eyre::Result<()> {
        let parameters = CreateSessionTransactionSignatureParameters {
            hash: fixed_bytes!(
                "0x438331d7eba6601df86b9ddc6b0ca3f5ec7ac0b395a3d7e2795fa2a855b4daad"
            ),
            to: address!("0xdebbd4ce2bd6bd869d3ac93666a0d5f4fc06fc72"),
            call_data: None,
            session_key: fixed_bytes!(
                "0x6954ddb21936036ccad688e2770846f15380a721bfab26c6e531e25b35cb5971"
            ),
            session_config:
                crate::utils::session::session_lib::session_spec::SessionSpec {
                    signer: address!(
                        "0x9bbc92a33f193174bf6cc09c4b4055500d972479"
                    ),
                    expires_at: U256::from(1767225600u64),
                    fee_limit: crate::utils::session::session_lib::session_spec::usage_limit::UsageLimit {
                        limit_type: crate::utils::session::session_lib::session_spec::limit_type::LimitType::Lifetime, // Lifetime
                        limit: U256::from(0u64),
                        period: U256::from(0u64),
                    },
                    call_policies: vec![],
                    transfer_policies: vec![
                        crate::utils::session::session_lib::session_spec::transfer_spec::TransferSpec {
                            target: address!("0xdebbd4ce2bd6bd869d3ac93666a0d5f4fc06fc72"),
                            max_value_per_use: U256::from(0u64),
                            value_limit: crate::utils::session::session_lib::session_spec::usage_limit::UsageLimit {
                                limit_type: crate::utils::session::session_lib::session_spec::limit_type::LimitType::Unlimited,
                                limit: U256::from(0u64),
                                period: U256::from(0u64),
                            },
                        },
                    ],
                },
            contracts: SSOContracts {
                account_factory: address!(
                    "0x0000000000000000000000000000000000000000"
                ),
                passkey: address!("0x0000000000000000000000000000000000000000"),
                session: address!("0x027ba0517cfa4471457c6e74f201753d98e7431d"),
                account_paymaster: address!(
                    "0x0000000000000000000000000000000000000000"
                ),
                recovery: address!(
                    "0x0000000000000000000000000000000000000000"
                ),
            },
            timestamp: Some(U64::from(1028u64)),
        };

        let custom_signature =
            create_session_transaction_signature(parameters)?;

        println!("custom_signature: {custom_signature:?}");

        let expected_custom_signature = bytes!(
            "0x0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000027ba0517cfa4471457c6e74f201753d98e7431d00000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000419bdec54176e0c91080f44c066837875a3c9afda7095568f64951b8ed9b6f420d7f9e89aa5d0be73a8625c41267bc4a7d62ac3fde218b5392b56cee2c23237e271b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000002000000000000000000000000009bbc92a33f193174bf6cc09c4b4055500d972479000000000000000000000000000000000000000000000000000000006955b9000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000debbd4ce2bd6bd869d3ac93666a0d5f4fc06fc72000000000000000000000000000000000000000000000000002386f26fc10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );

        eyre::ensure!(
            custom_signature == expected_custom_signature,
            "custom_signature does not match expected_custom_signature \
            custom_signature: {:?}, \
            expected_custom_signature: {:?}",
            custom_signature,
            expected_custom_signature,
        );

        Ok(())
    }

    fn create_mock_session_config() -> SessionSpec {
        SessionSpec {
            signer: address!("0x9BbC92a33F193174bf6Cc09c4b4055500d972479"),
            expires_at: U256::from(1749040108u64),
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(100000000000000000u64),
                period: U256::from(0),
            },
            call_policies: vec![CallSpec {
                target: address!("0x9876543210987654321098765432109876543210"),
                selector: fixed_bytes!("0xa9059cbb"),
                max_value_per_use: U256::from(20000000000000000u64),
                value_limit: UsageLimit {
                    limit_type: LimitType::Allowance,
                    limit: U256::from(50000000000000000u64),
                    period: U256::from(86400),
                },
                constraints: vec![],
            }],
            transfer_policies: vec![TransferSpec {
                target: address!("0x9876543210987654321098765432109876543210"),
                max_value_per_use: U256::from(20000000000000000u64),
                value_limit: UsageLimit {
                    limit_type: LimitType::Allowance,
                    limit: U256::from(50000000000000000u64),
                    period: U256::from(86400),
                },
            }],
        }
    }

    fn create_mock_contracts() -> SSOContracts {
        SSOContracts {
            account_factory: address!(
                "0x1234567890123456789012345678901234567890"
            ),
            session: address!("0x1234567890123456789012345678901234567890"),
            account_paymaster: address!(
                "0x2222222222222222222222222222222222222222"
            ),
            passkey: address!("0x3333333333333333333333333333333333333333"),
            recovery: address!("0x4444444444444444444444444444444444444444"),
        }
    }

    #[test]
    fn test_creates_session_transaction_signature_with_all_parameters_detailed_logging()
    -> eyre::Result<()> {
        println!(
            "\n=== CREATE SESSION TRANSACTION SIGNATURE - DETAILED BREAKDOWN ==="
        );

        // Test Vector 1 from TypeScript tests
        let mock_hash = fixed_bytes!(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        let mock_to = address!("0x9876543210987654321098765432109876543210");
        let mock_call_data: Option<Bytes> = Some(hex::decode(
            "a9059cbb000000000000000000000000742d35cc6481c2962f73827c58e76db6d23aa2d5000000000000000000000000000000000000000000000000016345785d8a0000"
        )?.into());
        let mock_session_key = fixed_bytes!(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        );
        let mock_timestamp = Some(U64::from(1749040108u64));
        let mock_contracts = create_mock_contracts();
        let mock_session_config = create_mock_session_config();

        println!("Input parameters:");
        println!("- hash: 0x{}", hex::encode(mock_hash));
        println!("- hash (bytes): {:?}", mock_hash.as_slice());
        println!("- to: {mock_to:?}");
        println!("- to (bytes): {:?}", mock_to.as_slice());
        if let Some(ref call_data) = mock_call_data {
            println!("- callData: 0x{}", hex::encode(call_data));
            println!("- callData (bytes): {:?}", call_data.as_ref());
        }
        println!("- sessionKey: 0x{}", hex::encode(mock_session_key));
        println!("- sessionKey (bytes): {:?}", mock_session_key.as_slice());
        if let Some(timestamp) = mock_timestamp {
            println!("- timestamp: {timestamp}");
        }
        println!("- contracts: {mock_contracts:?}");
        println!("\nSession Config: {mock_session_config:?}");

        let parameters = CreateSessionTransactionSignatureParameters {
            hash: mock_hash,
            to: mock_to,
            call_data: mock_call_data.clone(),
            session_key: mock_session_key,
            session_config: mock_session_config.clone(),
            contracts: mock_contracts,
            timestamp: mock_timestamp,
        };

        let result = create_session_transaction_signature(parameters).unwrap();

        println!("\n=== CREATE SESSION TRANSACTION SIGNATURE RESULT ===");
        println!("Session transaction signature: 0x{}", hex::encode(&result));
        println!(
            "Result length: {} characters",
            hex::encode(&result).len() + 2
        );
        println!("Result bytes length: {} bytes", result.len());
        println!("Result (bytes): {:?}", result.as_ref());

        // Break down the hex into chunks for easier analysis
        println!("\nHex breakdown (32-byte chunks):");
        let hex_without_prefix = hex::encode(&result);
        for (i, chunk) in hex_without_prefix.as_bytes().chunks(64).enumerate() {
            println!("Chunk {:02}: {}", i, String::from_utf8_lossy(chunk));
        }

        // Expected result from TypeScript test
        let expected_result = "0x0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000123456789012345678901234567890123456789000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000412cbd400991f55c3cde09d24d10d01bf48f7b2a319ef0e6f1a370b65bd4cfc57a4d7e69b218c9499831286c462155c7769345da24dd406d23ad5eaee540cecdea1b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000380000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000003200000000000000000000000009bbc92a33f193174bf6cc09c4b4055500d9724790000000000000000000000000000000000000000000000000000000068403bec0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000009876543210987654321098765432109876543210a9059cbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000470de4df820000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000b1a2bc2ec50000000000000000000000000000000000000000000000000000000000000001518000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000987654321098765432109876543210987654321000000000000000000000000000000000000000000000000000470de4df820000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000b1a2bc2ec500000000000000000000000000000000000000000000000000000000000000015180000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004f13";

        // Verify the result structure
        let result_hex = format!("0x{}", hex::encode(&result));
        assert!(result_hex.starts_with("0x"));
        assert!(result_hex.len() > 2);

        // Assert that the result matches the expected output exactly
        assert_eq!(
            result_hex, expected_result,
            "Generated session transaction signature does not match expected signature from TypeScript tests"
        );

        // Test deterministic behavior
        let parameters2 = CreateSessionTransactionSignatureParameters {
            hash: mock_hash,
            to: mock_to,
            call_data: mock_call_data,
            session_key: mock_session_key,
            session_config: mock_session_config.clone(),
            contracts: mock_contracts,
            timestamp: mock_timestamp,
        };

        let result2 =
            create_session_transaction_signature(parameters2).unwrap();
        assert_eq!(result, result2);
        println!("Deterministic test passed: same input produces same output");

        Ok(())
    }

    #[test]
    #[ignore = "this test is broken, need to investigate"]
    fn test_creates_session_transaction_signature_without_optional_parameters()
    -> eyre::Result<()> {
        println!(
            "\n=== CREATE SESSION TRANSACTION SIGNATURE WITHOUT OPTIONAL PARAMS ==="
        );

        let mock_hash = fixed_bytes!(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        let mock_to = address!("0x9876543210987654321098765432109876543210");
        let mock_session_key = fixed_bytes!(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        );
        let mock_contracts = create_mock_contracts();
        let mock_session_config = create_mock_session_config();

        println!("Input parameters (minimal):");
        println!("- hash: 0x{}", hex::encode(mock_hash));
        println!("- to: {mock_to:?}");
        println!("- sessionKey: 0x{}", hex::encode(mock_session_key));
        println!("- contracts: {mock_contracts:?}");

        let parameters = CreateSessionTransactionSignatureParameters {
            hash: mock_hash,
            to: mock_to,
            call_data: None, // callData omitted
            session_key: mock_session_key,
            session_config: mock_session_config,
            contracts: mock_contracts,
            timestamp: None, // timestamp omitted
        };

        let result = create_session_transaction_signature(parameters).unwrap();

        println!("Result without optional params: 0x{}", hex::encode(&result));
        println!(
            "Result length: {} characters",
            hex::encode(&result).len() + 2
        );

        // Expected result from TypeScript test (without optional params)
        let expected_result = "0x0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000123456789012345678901234567890123456789000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000412cbd400991f55c3cde09d24d10d01bf48f7b2a319ef0e6f1a370b65bd4cfc57a4d7e69b218c9499831286c462155c7769345da24dd406d23ad5eaee540cecdea1b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000380000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000003200000000000000000000000009bbc92a33f193174bf6cc09c4b4055500d9724790000000000000000000000000000000000000000000000000000000068403bec0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000009876543210987654321098765432109876543210a9059cbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000470de4df820000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000b1a2bc2ec50000000000000000000000000000000000000000000000000000000000000001518000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000987654321098765432109876543210987654321000000000000000000000000000000000000000000000000000470de4df820000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000b1a2bc2ec500000000000000000000000000000000000000000000000000000000000000015180000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004f23";

        // Should return a hex string
        let result_hex = format!("0x{}", hex::encode(&result));
        assert!(result_hex.starts_with("0x"));
        assert!(result_hex.len() > 2);

        // Assert that the result matches the expected output exactly
        assert_eq!(
            result_hex, expected_result,
            "Generated session transaction signature without optional params does not match expected signature from TypeScript tests"
        );

        // Should be different from the result with callData
        let parameters_with_call_data = CreateSessionTransactionSignatureParameters {
            hash: mock_hash,
            to: mock_to,
            call_data: Some(hex::decode(
                "a9059cbb000000000000000000000000742d35cc6481c2962f73827c58e76db6d23aa2d5000000000000000000000000000000000000000000000000016345785d8a0000"
            )?.into()),
            session_key: mock_session_key,
            session_config: create_mock_session_config(),
            contracts: create_mock_contracts(),
            timestamp: Some(U64::from(1749040108u64)),
        };

        let result_with_call_data =
            create_session_transaction_signature(parameters_with_call_data)
                .unwrap();

        assert_ne!(result, result_with_call_data);
        println!(
            "Results are different with/without callData: {}",
            result != result_with_call_data
        );
        Ok(())
    }

    #[test]
    fn test_produces_different_output_for_different_session_keys() {
        println!("\n=== DIFFERENT SESSION KEYS TEST ===");

        let session_key1 = fixed_bytes!(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        );
        let session_key2 = fixed_bytes!(
            "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
        );

        println!("Session key 1: 0x{}", hex::encode(session_key1));
        println!("Session key 2: 0x{}", hex::encode(session_key2));

        let mock_hash = fixed_bytes!(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        let mock_to = address!("0x9876543210987654321098765432109876543210");

        let parameters1 = CreateSessionTransactionSignatureParameters {
            hash: mock_hash,
            to: mock_to,
            call_data: None,
            session_key: session_key1,
            session_config: create_mock_session_config(),
            contracts: create_mock_contracts(),
            timestamp: None,
        };

        let parameters2 = CreateSessionTransactionSignatureParameters {
            hash: mock_hash,
            to: mock_to,
            call_data: None,
            session_key: session_key2,
            session_config: create_mock_session_config(),
            contracts: create_mock_contracts(),
            timestamp: None,
        };

        let result1 =
            create_session_transaction_signature(parameters1).unwrap();
        let result2 =
            create_session_transaction_signature(parameters2).unwrap();

        let result1_hex = format!("0x{}", hex::encode(&result1));
        let result2_hex = format!("0x{}", hex::encode(&result2));

        println!("Result with key 1: {result1_hex}");
        println!("Result with key 2: {result2_hex}");
        println!("Results are different: {}", result1 != result2);

        assert_ne!(result1, result2);
    }

    #[test]
    fn test_produces_different_output_for_different_hashes() {
        println!("\n=== DIFFERENT HASHES TEST ===");

        let hash1 = fixed_bytes!(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        let hash2 = fixed_bytes!(
            "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"
        );

        println!("Hash 1: 0x{}", hex::encode(hash1));
        println!("Hash 2: 0x{}", hex::encode(hash2));

        let mock_to = address!("0x9876543210987654321098765432109876543210");
        let mock_session_key = fixed_bytes!(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        );

        let parameters1 = CreateSessionTransactionSignatureParameters {
            hash: hash1,
            to: mock_to,
            call_data: None,
            session_key: mock_session_key,
            session_config: create_mock_session_config(),
            contracts: create_mock_contracts(),
            timestamp: None,
        };

        let parameters2 = CreateSessionTransactionSignatureParameters {
            hash: hash2,
            to: mock_to,
            call_data: None,
            session_key: mock_session_key,
            session_config: create_mock_session_config(),
            contracts: create_mock_contracts(),
            timestamp: None,
        };

        let result1 =
            create_session_transaction_signature(parameters1).unwrap();
        let result2 =
            create_session_transaction_signature(parameters2).unwrap();

        let result1_hex = format!("0x{}", hex::encode(&result1));
        let result2_hex = format!("0x{}", hex::encode(&result2));

        println!("Result with hash 1: {result1_hex}");
        println!("Result with hash 2: {result2_hex}");
        println!("Results are different: {}", result1 != result2);

        assert_ne!(result1, result2);
    }

    #[test]
    fn test_handles_complex_session_configuration() -> eyre::Result<()> {
        println!("\n=== COMPLEX SESSION CONFIG TEST ===");

        let complex_session_config = SessionSpec {
            signer: address!("0x1111111111111111111111111111111111111111"),
            expires_at: U256::from(2000000000u64),
            fee_limit: UsageLimit {
                limit_type: LimitType::Allowance,
                limit: U256::from(200000000000000000u64),
                period: U256::from(3600),
            },
            call_policies: vec![CallSpec {
                target: address!("0x2222222222222222222222222222222222222222"),
                selector: fixed_bytes!("0xa9059cbb"),
                max_value_per_use: U256::from(1000000000000000u64),
                value_limit: UsageLimit {
                    limit_type: LimitType::Lifetime,
                    limit: U256::from(10000000000000000u64),
                    period: U256::from(0),
                },
                constraints: vec![Constraint {
                    index: 4u64,
                    condition: Condition::Equal,
                    ref_value: fixed_bytes!(
                        "0x0000000000000000000000000000000000000000000000000000000000000000"
                    ),
                    limit: UsageLimit {
                        limit_type: LimitType::Allowance,
                        limit: U256::from(5000000000000000u64),
                        period: U256::from(1800),
                    },
                }],
            }],
            transfer_policies: vec![TransferSpec {
                target: address!("0x3333333333333333333333333333333333333333"),
                max_value_per_use: U256::from(30000000000000000u64),
                value_limit: UsageLimit {
                    limit_type: LimitType::Allowance,
                    limit: U256::from(100000000000000000u64),
                    period: U256::from(604800),
                },
            }],
        };

        println!("Complex session config: {complex_session_config:?}");

        let mock_hash = fixed_bytes!(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        let mock_to = address!("0x2222222222222222222222222222222222222222");
        let mock_call_data: Option<Bytes> = Some(hex::decode(
            "a9059cbb000000000000000000000000742d35cc6481c2962f73827c58e76db6d23aa2d5000000000000000000000000000000000000000000000000016345785d8a0000"
        )?.into());
        let mock_session_key = fixed_bytes!(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        );
        let mock_timestamp = Some(U64::from(2000000000u64));

        let parameters = CreateSessionTransactionSignatureParameters {
            hash: mock_hash,
            to: mock_to,
            call_data: mock_call_data,
            session_key: mock_session_key,
            session_config: complex_session_config,
            contracts: create_mock_contracts(),
            timestamp: mock_timestamp,
        };

        let result = create_session_transaction_signature(parameters).unwrap();

        println!("Result with complex config: 0x{}", hex::encode(&result));
        println!(
            "Result length: {} characters",
            hex::encode(&result).len() + 2
        );

        let result_hex = format!("0x{}", hex::encode(&result));
        assert!(result_hex.starts_with("0x"));
        assert!(result_hex.len() > 2);

        // Should be different from simple session config
        let simple_parameters = CreateSessionTransactionSignatureParameters {
            hash: mock_hash,
            to: address!("0x9876543210987654321098765432109876543210"),
            call_data: None,
            session_key: mock_session_key,
            session_config: create_mock_session_config(),
            contracts: create_mock_contracts(),
            timestamp: None,
        };

        let simple_result =
            create_session_transaction_signature(simple_parameters).unwrap();

        assert_ne!(result, simple_result);
        println!(
            "Complex config produces different result: {}",
            result != simple_result
        );
        Ok(())
    }
}
