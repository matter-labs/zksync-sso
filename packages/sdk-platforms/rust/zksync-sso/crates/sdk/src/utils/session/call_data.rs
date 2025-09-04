#[cfg(test)]
mod tests {
    use crate::{
        config::Config,
        contracts::{
            SessionKeyValidator,
            SessionLib::{
                CallSpec, Constraint, SessionSpec, TransferSpec, UsageLimit,
            },
        },
        utils::session::session_lib::session_spec::limit_type::LimitType,
    };
    use alloy::{
        primitives::{
            Address, Bytes, FixedBytes, U256, address, fixed_bytes, hex,
        },
        sol_types::SolCall,
    };
    use alloy_zksync::provider::zksync_provider;
    use url;

    pub(crate) fn generate_session_call_data(
        session_spec: SessionSpec,
    ) -> Bytes {
        let call = SessionKeyValidator::createSessionCall {
            sessionSpec: session_spec,
        };
        let encoded_call = call.abi_encode();
        let encoded_call_bytes: Bytes = encoded_call.into();
        encoded_call_bytes
    }

    pub(crate) fn create_session_call_data(
        session_key_module_address: Address,
        session_spec: SessionSpec,
        config: &Config,
    ) -> Bytes {
        let provider = {
            let node_url: url::Url = config.clone().node_url;

            zksync_provider()
                .with_recommended_fillers()
                .on_http(node_url.clone())
        };

        let session_validator =
            SessionKeyValidator::new(session_key_module_address, &provider);

        session_validator.createSession(session_spec).calldata().to_owned()
    }

    /// Creates a SessionSpec that matches the detailed test case from TypeScript
    fn create_detailed_test_session_spec() -> SessionSpec {
        SessionSpec {
            signer: address!("0x9BbC92a33F193174bf6Cc09c4b4055500d972479"),
            expiresAt: U256::from(1749040108u64),
            feeLimit: UsageLimit {
                limitType: LimitType::Allowance as u8, // limitType: 2 in TS
                limit: U256::from(100000000000000000u64), // parseEther("0.1")
                period: U256::ZERO,
            },
            callPolicies: vec![CallSpec {
                target: address!("0x1111111111111111111111111111111111111111"),
                selector: fixed_bytes!("0xa9059cbb"), // transfer function selector
                maxValuePerUse: U256::from(1000000000000000u64), // parseEther("0.001")
                valueLimit: UsageLimit {
                    limitType: LimitType::Allowance as u8, // limitType: 2 in TS
                    limit: U256::from(10000000000000000u64), // parseEther("0.01")
                    period: U256::ZERO,
                },
                constraints: vec![Constraint {
                    condition: 1, // Condition::Equal
                    index: 4,
                    refValue: FixedBytes::ZERO, // ethers.ZeroHash
                    limit: UsageLimit {
                        limitType: LimitType::Allowance as u8, // limitType: 2 in TS
                        limit: U256::from(5000000000000000u64), // parseEther("0.005")
                        period: U256::from(1800u64),            // 30 minutes
                    },
                }],
            }],
            transferPolicies: vec![TransferSpec {
                target: address!("0x0000000000000000000000000000000000000000"), // ethers.ZeroAddress
                maxValuePerUse: U256::from(20000000000000000u64), // parseEther("0.02")
                valueLimit: UsageLimit {
                    limitType: LimitType::Lifetime as u8, // limitType: 1 in TS
                    limit: U256::ZERO,
                    period: U256::ZERO,
                },
            }],
        }
    }

    /// Creates a minimal SessionSpec for testing different configs
    fn create_minimal_session_spec() -> SessionSpec {
        SessionSpec {
            signer: address!("0x1111111111111111111111111111111111111111"),
            expiresAt: U256::from(1749040108u64),
            feeLimit: UsageLimit {
                limitType: LimitType::Lifetime as u8, // limitType: 1 in TS
                limit: U256::from(100000000000000000u64), // parseEther("0.1")
                period: U256::ZERO,
            },
            callPolicies: vec![],
            transferPolicies: vec![],
        }
    }

    /// Creates a modified version of the minimal session spec with different signer
    fn create_modified_session_spec() -> SessionSpec {
        SessionSpec {
            signer: address!("0x2222222222222222222222222222222222222222"),
            expiresAt: U256::from(1749040108u64),
            feeLimit: UsageLimit {
                limitType: LimitType::Lifetime as u8, // limitType: 1 in TS
                limit: U256::from(100000000000000000u64), // parseEther("0.1")
                period: U256::ZERO,
            },
            callPolicies: vec![],
            transferPolicies: vec![],
        }
    }

    #[test]
    fn test_generates_call_data_for_session_creation_with_detailed_logging() {
        println!("\n=== GENERATE SESSION CALL DATA - DETAILED BREAKDOWN ===");

        // Create the same session config as in the TypeScript test
        let test_session_config = create_detailed_test_session_spec();

        println!("Input SessionConfig created (Rust equivalent)");

        // Generate the call data
        let call_data = generate_session_call_data(test_session_config.clone());

        println!("\n=== CALL DATA RESULT ===");
        println!("Generated call data: 0x{}", hex::encode(&call_data));
        println!(
            "Call data length: {} characters",
            format!("0x{}", hex::encode(&call_data)).len()
        );
        println!("Call data bytes length: {} bytes", call_data.len());

        // Break down the hex into chunks for easier analysis
        println!("\nHex breakdown (32-byte chunks):");
        let hex_string = hex::encode(&call_data);
        for (i, chunk) in hex_string.as_bytes().chunks(64).enumerate() {
            let chunk_str = std::str::from_utf8(chunk).unwrap();
            println!("Chunk {i:02}: {chunk_str}");
        }

        // Expected function selector for createSession
        let expected_selector = "5a0694d2"; // createSession function selector from TS output
        println!("\nFunction analysis:");
        println!("Expected createSession selector: 0x{expected_selector}");
        println!("Actual function selector: 0x{}", &hex_string[..8]);
        println!("Selector matches: {}", &hex_string[..8] == expected_selector);

        // Verify the call data starts with the correct function selector
        let call_data_hex = format!("0x{}", hex::encode(&call_data));
        assert!(call_data_hex.chars().skip(2).all(|c| c.is_ascii_hexdigit()));
        assert_eq!(&call_data_hex[2..10], expected_selector);
        assert!(call_data_hex.len() > 10);

        // Test deterministic behavior
        let call_data2 = generate_session_call_data(test_session_config);
        assert_eq!(call_data, call_data2);
        println!("Deterministic test passed: same input produces same output");

        // Expected call data from TypeScript test output
        let expected_call_data = "0x5a0694d200000000000000000000000000000000000000000000000000000000000000200000000000000000000000009bbc92a33f193174bf6cc09c4b4055500d9724790000000000000000000000000000000000000000000000000000000068403bec0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000002e0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000001111111111111111111111111111111111111111a9059cbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000038d7ea4c680000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000002386f26fc10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000011c37937e0800000000000000000000000000000000000000000000000000000000000000007080000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000470de4df820000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

        // Verify the generated call data matches the expected output from TypeScript
        assert_eq!(
            call_data_hex, expected_call_data,
            "Generated call data does not match TypeScript test expected output"
        );

        println!("✅ Call data matches TypeScript test expected output");
    }

    #[test]
    fn test_generates_different_call_data_for_different_session_configs() {
        let base_config = create_minimal_session_spec();
        let modified_config = create_modified_session_spec();

        let call_data1 = generate_session_call_data(base_config);
        let call_data2 = generate_session_call_data(modified_config);

        println!("\n=== DIFFERENT CONFIG TEST ===");
        println!("Base config call data: 0x{}", hex::encode(&call_data1));
        println!("Modified config call data: 0x{}", hex::encode(&call_data2));
        println!("Call data are different: {}", call_data1 != call_data2);

        assert_ne!(call_data1, call_data2);

        // Both should have the same function selector
        let hex1 = hex::encode(&call_data1);
        let hex2 = hex::encode(&call_data2);
        assert_eq!(&hex1[..8], &hex2[..8]); // Same function selector

        // Expected call data from TypeScript test for base config
        let expected_base_call_data = "0x5a0694d2000000000000000000000000000000000000000000000000000000000000002000000000000000000000000011111111111111111111111111111111111111110000000000000000000000000000000000000000000000000000000068403bec0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

        // Expected call data from TypeScript test for modified config
        let expected_modified_call_data = "0x5a0694d2000000000000000000000000000000000000000000000000000000000000002000000000000000000000000022222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000068403bec0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

        let call_data1_hex = format!("0x{}", hex::encode(&call_data1));
        let call_data2_hex = format!("0x{}", hex::encode(&call_data2));

        // Verify against TypeScript expected outputs
        assert_eq!(
            call_data1_hex, expected_base_call_data,
            "Base config call data does not match TypeScript expected output"
        );

        assert_eq!(
            call_data2_hex, expected_modified_call_data,
            "Modified config call data does not match TypeScript expected output"
        );

        println!("✅ Both call data match TypeScript test expected outputs");
    }

    #[test]
    fn test_function_selector_consistency() {
        // Test that all generated call data starts with the same function selector
        let configs = vec![
            create_detailed_test_session_spec(),
            create_minimal_session_spec(),
            create_modified_session_spec(),
        ];

        let expected_selector = "5a0694d2";

        for (i, config) in configs.into_iter().enumerate() {
            let call_data = generate_session_call_data(config);
            let hex_string = hex::encode(&call_data);

            assert_eq!(
                &hex_string[..8],
                expected_selector,
                "Config {i} does not have the expected function selector"
            );
        }
    }

    #[test]
    fn test_empty_policies_encoding() {
        // Test a session config with no call policies or transfer policies
        let session_config = SessionSpec {
            signer: address!("0x1111111111111111111111111111111111111111"),
            expiresAt: U256::from(1749040108u64),
            feeLimit: UsageLimit {
                limitType: LimitType::Lifetime as u8,
                limit: U256::from(100000000000000000u64),
                period: U256::ZERO,
            },
            callPolicies: vec![],
            transferPolicies: vec![],
        };

        let call_data = generate_session_call_data(session_config);
        let hex_string = hex::encode(&call_data);

        // Should still have the correct function selector
        assert_eq!(&hex_string[..8], "5a0694d2");

        // Should be deterministic
        let call_data2 = generate_session_call_data(SessionSpec {
            signer: address!("0x1111111111111111111111111111111111111111"),
            expiresAt: U256::from(1749040108u64),
            feeLimit: UsageLimit {
                limitType: LimitType::Lifetime as u8,
                limit: U256::from(100000000000000000u64),
                period: U256::ZERO,
            },
            callPolicies: vec![],
            transferPolicies: vec![],
        });

        assert_eq!(call_data, call_data2);
    }

    #[test]
    fn test_call_data_is_valid_hex() {
        let session_config = create_detailed_test_session_spec();
        let call_data = generate_session_call_data(session_config);
        let hex_string = hex::encode(&call_data);

        // Should be valid hex characters only
        assert!(hex_string.chars().all(|c| c.is_ascii_hexdigit()));

        // Should have even length (each byte represented by 2 hex chars)
        assert_eq!(hex_string.len() % 2, 0);

        // Should be at least 8 characters (4 bytes for function selector)
        assert!(hex_string.len() >= 8);
    }

    #[test]
    fn test_generate_vs_create_session_call_data_consistency() {
        use crate::config::{
            Config, contracts::SSOContracts, deploy_wallet::DeployWallet,
        };
        use alloy::primitives::address;
        use url::Url;

        // Create a mock config for the create_session_call_data function
        let mock_config = Config {
            node_url: Url::parse("http://localhost:8011").unwrap(),
            deploy_wallet: Some(DeployWallet {
                private_key_hex: "0x3d3cbc973389cb26f657686445bcc75662b415b656078503592ac8c1abb8810e".to_string(),
            }),
            contracts: SSOContracts {
                account_factory: address!("0x1234567890123456789012345678901234567890"),
                passkey: address!("0x3333333333333333333333333333333333333333"),
                session: address!("0x1111111111111111111111111111111111111111"),
                account_paymaster: address!("0x2222222222222222222222222222222222222222"),
                recovery: address!("0x4444444444444444444444444444444444444444"),
            },
        };

        // Create test session specs for comparison
        let test_cases = vec![
            ("detailed_session", create_detailed_test_session_spec()),
            ("minimal_session", create_minimal_session_spec()),
            ("modified_session", create_modified_session_spec()),
        ];

        for (test_name, session_spec) in test_cases {
            println!("\n=== COMPARING FUNCTIONS FOR {test_name} ===");

            // Call generate_session_call_data (our implementation)
            let generate_result =
                generate_session_call_data(session_spec.clone());

            // Call create_session_call_data (the existing implementation)
            let create_result = create_session_call_data(
                mock_config.contracts.session,
                session_spec.clone(),
                &mock_config,
            );

            println!(
                "generate_session_call_data result: 0x{}",
                hex::encode(&generate_result)
            );
            println!(
                "create_session_call_data result:   0x{}",
                hex::encode(&create_result)
            );
            println!("Results match: {}", generate_result == create_result);

            // Assert that both functions produce identical results
            assert_eq!(
                generate_result, create_result,
                "Functions should produce identical call data for {test_name}"
            );

            // Additional verification: both should have the same function selector
            let generate_hex = hex::encode(&generate_result);
            let create_hex = hex::encode(&create_result);
            assert_eq!(
                &generate_hex[..8],
                &create_hex[..8],
                "Both functions should have the same function selector for {test_name}"
            );

            println!(
                "✅ Both functions produce identical results for {test_name}"
            );
        }

        println!("\n🎉 All function comparison tests passed!");
    }
}
