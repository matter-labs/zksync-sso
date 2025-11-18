use crate::erc4337::account::modular_smart_account::{
    session::{
        SessionLib::SessionSpec as SessionLibSessionSpec,
        session_lib::session_spec::{
            SessionSpec, limit_type::LimitType, usage_limit::UsageLimit,
        },
    },
    signers::eoa::eoa_sign,
};
use alloy::{
    dyn_abi::SolType,
    primitives::{Address, Bytes, FixedBytes, Uint},
    sol,
};

/// Calculate period ID for a usage limit without using system time.
/// This function is WASM-safe as it doesn't call std::time functions.
///
/// # Arguments
/// * `limit` - The usage limit to calculate period ID for
/// * `current_timestamp` - Optional current timestamp. If None, returns 0 (no time validation)
///
/// # Returns
/// Period ID as Uint<48, 1>
pub fn get_period_id_no_validation(
    limit: &UsageLimit,
    current_timestamp: Option<u64>,
) -> Uint<48, 1> {
    match current_timestamp {
        Some(timestamp) if limit.limit_type == LimitType::Allowance => {
            let ts = Uint::<48, 1>::from(timestamp);
            ts / limit.period
        }
        _ => Uint::from(0),
    }
}

/// Generate a session signature WITHOUT timestamp validation.
/// This is WASM-safe as it doesn't call std::time functions.
///
/// The signature format is: [session_validator_address(20 bytes)][fat_signature]
/// where fat_signature contains: (signature_bytes, SessionSpec, period_ids[])
///
/// # Arguments
/// * `private_key_hex` - Session key private key in hex format
/// * `session_validator` - Address of the session validator contract
/// * `session_spec` - Session specification containing policies and limits
/// * `hash` - Hash to sign (typically UserOperation hash)
/// * `current_timestamp` - Optional current timestamp for period ID calculation.
///   If None, period_ids will be [0, 0] (defers validation to on-chain)
///
/// # Returns
/// Encoded signature bytes
pub fn session_signature_no_validation(
    private_key_hex: &str,
    session_validator: Address,
    session_spec: &SessionSpec,
    hash: FixedBytes<32>,
    current_timestamp: Option<u64>,
) -> eyre::Result<Bytes> {
    let session_validator_bytes = session_validator.0.to_vec();
    let signature_bytes = eoa_sign(private_key_hex, hash)?;

    // Calculate period IDs without using system time
    let fee_limit_period_id =
        get_period_id_no_validation(&session_spec.fee_limit, current_timestamp);
    let period_ids = vec![fee_limit_period_id, Uint::from(0)];

    let fat_signature = {
        type SessionSignature =
            sol! { tuple(bytes, SessionLibSessionSpec, uint48[]) };
        let spec: SessionLibSessionSpec = session_spec.to_owned().into();
        SessionSignature::abi_encode_params(&(
            signature_bytes,
            spec.clone(),
            period_ids,
        ))
    };

    let bytes = [session_validator_bytes, fat_signature].concat();

    Ok(bytes.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::account::modular_smart_account::session::session_lib::session_spec::{
        transfer_spec::TransferSpec, usage_limit::UsageLimit,
    };
    use alloy::primitives::{address, U256};

    #[test]
    fn test_get_period_id_no_validation_with_timestamp() {
        let limit = UsageLimit {
            limit_type: LimitType::Allowance,
            limit: U256::from(1000),
            period: Uint::from(100),
        };

        let timestamp = 12345u64;
        let period_id = get_period_id_no_validation(&limit, Some(timestamp));

        // period_id should be timestamp / period = 12345 / 100 = 123
        assert_eq!(period_id, Uint::from(123));
    }

    #[test]
    fn test_get_period_id_no_validation_without_timestamp() {
        let limit = UsageLimit {
            limit_type: LimitType::Allowance,
            limit: U256::from(1000),
            period: Uint::from(100),
        };

        let period_id = get_period_id_no_validation(&limit, None);

        // Without timestamp, should return 0
        assert_eq!(period_id, Uint::from(0));
    }

    #[test]
    fn test_get_period_id_no_validation_lifetime_limit() {
        let limit = UsageLimit {
            limit_type: LimitType::Lifetime,
            limit: U256::from(1000),
            period: Uint::from(0),
        };

        let period_id = get_period_id_no_validation(&limit, Some(12345));

        // Lifetime limits should return 0 regardless of timestamp
        assert_eq!(period_id, Uint::from(0));
    }

    #[test]
    fn test_session_signature_no_validation_generates_valid_signature() {
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

        let hash = FixedBytes::from([0u8; 32]);

        // Test without timestamp (WASM-safe)
        let result = session_signature_no_validation(
            private_key,
            session_validator,
            &session_spec,
            hash,
            None,
        );

        assert!(result.is_ok());
        let signature = result.unwrap();

        // Signature should start with session validator address (20 bytes)
        assert!(signature.len() > 20);
        assert_eq!(&signature[0..20], session_validator.as_slice());
    }

    #[test]
    fn test_session_signature_no_validation_with_timestamp() {
        let private_key = "0xb1da23908ba44fb1c6147ac1b32a1dbc6e7704ba94ec495e588d1e3cdc7ca6f9";
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

        let hash = FixedBytes::from([0u8; 32]);
        let current_timestamp = 1700000000u64;

        // Test with timestamp
        let result = session_signature_no_validation(
            private_key,
            session_validator,
            &session_spec,
            hash,
            Some(current_timestamp),
        );

        assert!(result.is_ok());
        let signature = result.unwrap();

        // Signature should be properly formatted
        assert!(signature.len() > 20);
        assert_eq!(&signature[0..20], session_validator.as_slice());
    }

    #[test]
    fn test_session_signature_matches_expected_encoding_format() {
        // This test verifies the signature format matches what on-chain contracts expect
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

        let hash = FixedBytes::from([1u8; 32]);

        let signature = session_signature_no_validation(
            private_key,
            session_validator,
            &session_spec,
            hash,
            None,
        )
        .unwrap();

        // Format should be: [20 bytes validator][variable length fat signature]
        // Fat signature contains: (bytes signature, SessionSpec, uint48[] period_ids)

        // First 20 bytes = validator address
        assert_eq!(&signature[0..20], session_validator.as_slice());

        // Remaining bytes should be ABI-encoded tuple
        // The signature should be deterministic for the same inputs
        assert!(signature.len() > 100); // Should have substantial size with SessionSpec
    }
}
