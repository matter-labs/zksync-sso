use crate::erc4337::account::{
    erc7579::executeCall,
    modular_smart_account::session::{
        SessionLib::SessionSpec as SessionLibSessionSpec,
        session_lib::session_spec::SessionSpec,
        signature_wasm::get_period_id_no_validation,
    },
};
use alloy::{
    dyn_abi::SolType,
    primitives::{Address, Bytes, FixedBytes, U256, Uint},
    sol,
    sol_types::SolCall,
};

/// Encode execute call data for a session transaction.
/// This is similar to the standard execute encoding but used specifically for session transactions.
///
/// # Arguments
/// * `target` - Target contract address
/// * `value` - ETH value to send
/// * `data` - Call data
///
/// # Returns
/// Encoded call data for execute(bytes32 mode, bytes memory execution)
pub fn encode_session_user_operation(
    target: Address,
    value: U256,
    data: Bytes,
) -> Bytes {
    // Create single execution mode (0x01 in first byte)
    let mode = {
        let mut mode_bytes = [0u8; 32];
        mode_bytes[0] = 1;
        FixedBytes::from(mode_bytes)
    };

    // Pack execution: target (20 bytes) + value (32 bytes) + data (variable)
    let execution: Bytes = [
        target.as_slice(),
        value
            .as_le_bytes()
            .iter()
            .rev()
            .copied()
            .collect::<Vec<_>>()
            .as_slice(),
        data.to_vec().as_slice(),
    ]
    .concat()
    .into();

    executeCall { mode, execution }.abi_encode().into()
}

/// Generate a stub signature for gas estimation.
/// This creates a properly-sized signature that matches the format of a real session signature
/// but doesn't contain actual cryptographic data.
///
/// The stub signature format matches session_signature_no_validation output:
/// [session_validator_address(20 bytes)][fat_signature]
///
/// # Arguments
/// * `session_validator` - Address of the session validator contract
/// * `session_spec` - Session specification
/// * `current_timestamp` - Optional timestamp for period ID calculation (for accurate size estimation)
///
/// # Returns
/// Stub signature bytes of the correct size for gas estimation
pub fn generate_session_stub_signature(
    session_validator: Address,
    session_spec: &SessionSpec,
    current_timestamp: Option<u64>,
) -> Bytes {
    let session_validator_bytes = session_validator.0.to_vec();

    // Create a stub ECDSA signature (65 bytes: r(32) + s(32) + v(1))
    let stub_ecdsa_signature = vec![0u8; 65];

    // Calculate period IDs (same logic as in session_signature_no_validation)
    let fee_limit_period_id =
        get_period_id_no_validation(&session_spec.fee_limit, current_timestamp);
    let period_ids = vec![fee_limit_period_id, Uint::from(0)];

    // Create the fat signature with the stub ECDSA signature
    let fat_signature = {
        type SessionSignature =
            sol! { tuple(bytes, SessionLibSessionSpec, uint48[]) };
        let spec: SessionLibSessionSpec = session_spec.to_owned().into();
        SessionSignature::abi_encode_params(&(
            Bytes::from(stub_ecdsa_signature),
            spec.clone(),
            period_ids,
        ))
    };

    let bytes = [session_validator_bytes, fat_signature].concat();

    bytes.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::account::modular_smart_account::session::session_lib::session_spec::{
        limit_type::LimitType, transfer_spec::TransferSpec,
        usage_limit::UsageLimit,
    };
    use alloy::primitives::address;

    #[test]
    fn test_encode_session_user_operation() {
        let target = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
        let value = U256::from(1000);
        let data = Bytes::from(vec![1, 2, 3, 4]);

        let encoded =
            encode_session_user_operation(target, value, data.clone());

        // Should be ABI-encoded execute(bytes32 mode, bytes memory execution) call
        assert!(!encoded.is_empty());

        // Verify the encoding matches executeCall format
        let mode = {
            let mut mode_bytes = [0u8; 32];
            mode_bytes[0] = 1;
            FixedBytes::from(mode_bytes)
        };

        let execution: Bytes = [
            target.as_slice(),
            value
                .as_le_bytes()
                .iter()
                .rev()
                .copied()
                .collect::<Vec<_>>()
                .as_slice(),
            data.to_vec().as_slice(),
        ]
        .concat()
        .into();

        let expected = executeCall { mode, execution };
        let expected_encoded: Bytes = expected.abi_encode().into();

        assert_eq!(encoded, expected_encoded);
    }

    #[test]
    fn test_encode_session_user_operation_empty_data() {
        let target = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
        let value = U256::from(1000);
        let data = Bytes::default();

        let encoded = encode_session_user_operation(target, value, data);

        // Should still encode properly with empty data
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_generate_session_stub_signature_has_correct_format() {
        let session_validator =
            address!("0x1234567890123456789012345678901234567890");
        let session_signer =
            address!("0xCEbb58e4082Af6FaC6Ea275740f10073d1610ad9");

        let session_spec = SessionSpec {
            signer: session_signer,
            expires_at: Uint::from(2088558400u64),
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(1_000_000_000_000_000_000u64),
                period: Uint::from(0),
            },
            call_policies: vec![],
            transfer_policies: vec![TransferSpec {
                target: address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720"),
                max_value_per_use: U256::from(1),
                value_limit: UsageLimit {
                    limit_type: LimitType::Unlimited,
                    limit: U256::from(0),
                    period: Uint::from(0),
                },
            }],
        };

        let stub_sig = generate_session_stub_signature(
            session_validator,
            &session_spec,
            None,
        );

        // Should start with session validator address (20 bytes)
        assert!(stub_sig.len() > 20);
        assert_eq!(&stub_sig[0..20], session_validator.as_slice());

        // Should contain a properly sized signature
        // Minimum size: 20 (validator) + 65 (ECDSA sig) + encoded SessionSpec + encoded period_ids
        assert!(stub_sig.len() > 100);
    }

    #[test]
    fn test_generate_session_stub_signature_with_timestamp() {
        let session_validator =
            address!("0x1234567890123456789012345678901234567890");
        let session_signer =
            address!("0xCEbb58e4082Af6FaC6Ea275740f10073d1610ad9");

        let session_spec = SessionSpec {
            signer: session_signer,
            expires_at: Uint::from(2088558400u64),
            fee_limit: UsageLimit {
                limit_type: LimitType::Allowance,
                limit: U256::from(1_000_000_000_000_000_000u64),
                period: Uint::from(86400), // 1 day
            },
            call_policies: vec![],
            transfer_policies: vec![],
        };

        let current_timestamp = 1700000000u64;

        let stub_sig = generate_session_stub_signature(
            session_validator,
            &session_spec,
            Some(current_timestamp),
        );

        // Should generate valid stub signature with timestamp
        assert!(stub_sig.len() > 20);
        assert_eq!(&stub_sig[0..20], session_validator.as_slice());
    }

    #[test]
    fn test_generate_session_stub_signature_size_consistency() {
        let session_validator =
            address!("0x1234567890123456789012345678901234567890");
        let session_signer =
            address!("0xCEbb58e4082Af6FaC6Ea275740f10073d1610ad9");

        let session_spec = SessionSpec {
            signer: session_signer,
            expires_at: Uint::from(2088558400u64),
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(1_000_000_000_000_000_000u64),
                period: Uint::from(0),
            },
            call_policies: vec![],
            transfer_policies: vec![],
        };

        // Generate stub signature multiple times - should be consistent size
        let stub1 = generate_session_stub_signature(
            session_validator,
            &session_spec,
            None,
        );
        let stub2 = generate_session_stub_signature(
            session_validator,
            &session_spec,
            None,
        );

        assert_eq!(stub1.len(), stub2.len());
    }

    #[test]
    fn test_stub_signature_matches_real_signature_size() {
        // This test verifies that the stub signature size closely matches
        // what a real signature would be, which is important for accurate gas estimation

        use crate::erc4337::account::modular_smart_account::session::signature_wasm::session_signature_no_validation;
        use alloy::primitives::FixedBytes;

        let private_key = "0xb1da23908ba44fb1c6147ac1b32a1dbc6e7704ba94ec495e588d1e3cdc7ca6f9";
        let session_validator =
            address!("0x1234567890123456789012345678901234567890");
        let session_signer =
            address!("0xCEbb58e4082Af6FaC6Ea275740f10073d1610ad9");

        let session_spec = SessionSpec {
            signer: session_signer,
            expires_at: Uint::from(2088558400u64),
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(1_000_000_000_000_000_000u64),
                period: Uint::from(0),
            },
            call_policies: vec![],
            transfer_policies: vec![],
        };

        let hash = FixedBytes::from([0u8; 32]);

        // Generate real signature
        let real_sig = session_signature_no_validation(
            private_key,
            session_validator,
            &session_spec,
            hash,
            None,
        )
        .unwrap();

        // Generate stub signature
        let stub_sig = generate_session_stub_signature(
            session_validator,
            &session_spec,
            None,
        );

        // Sizes should match exactly (same encoding structure)
        assert_eq!(
            real_sig.len(),
            stub_sig.len(),
            "Stub signature size mismatch: stub={}, real={}",
            stub_sig.len(),
            real_sig.len()
        );
    }
}
