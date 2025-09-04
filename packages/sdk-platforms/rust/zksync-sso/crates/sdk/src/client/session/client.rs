use crate::utils::{
    encoding::session::transaction::encode_session_tx,
    session::session_lib::session_spec::SessionSpec,
};
use alloy::{
    primitives::{Address, Bytes, U64},
    sol,
    sol_types::SolType,
};
use log::debug;

pub mod session_client;

sol! {
    struct SignSessionTxParams {
        bytes sessionKeySignedHash;
        address sessionContract;
        bytes validatorData;
    }
}

#[derive(Debug, Clone)]
pub struct SessionTransactionEncodedParamsArgs {
    pub session_key_signed_hash: Bytes,
    pub session_contract: Address,
    pub session_config: SessionSpec,
    pub to: Address,
    pub call_data: Option<Bytes>,
    pub timestamp: Option<U64>,
}

pub fn encoded_session_transaction_signature(
    args: SessionTransactionEncodedParamsArgs,
) -> eyre::Result<Bytes> {
    debug!("XDB - sign_session_transaction - args: {args:?}");

    let session_key_signed_hash = args.session_key_signed_hash;
    let session_contract = args.session_contract;
    let session_config = args.session_config;
    let to = args.to;
    let call_data = args.call_data;
    let timestamp = args.timestamp;

    let validator_data =
        encode_session_tx(session_config, to, call_data, timestamp)?;

    let params = SignSessionTxParams {
        sessionKeySignedHash: session_key_signed_hash,
        sessionContract: session_contract,
        validatorData: validator_data,
    };

    let params_bytes =
        <SignSessionTxParams as SolType>::abi_encode_params(&params);

    debug!("XDB - sign_session_transaction - params_bytes: {params_bytes:?}");

    Ok(params_bytes.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::session::session_lib::session_spec::{
        SessionSpec, call_spec::CallSpec, limit_type::LimitType,
        transfer_spec::TransferSpec, usage_limit::UsageLimit,
    };
    use alloy::primitives::{FixedBytes, U256, address, hex};

    #[test]
    fn test_sign_session_transaction_with_all_parameters() -> eyre::Result<()> {
        // Mock data matching the TypeScript test exactly
        let mock_session_key_signed_hash = hex::decode(
            "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
        )?;
        let mock_session_contract =
            address!("0x1234567890123456789012345678901234567890");
        let mock_to = address!("0x9876543210987654321098765432109876543210");
        let mock_call_data = Some(hex::decode(
            "a9059cbb000000000000000000000000742d35cc6481c2962f73827c58e76db6d23aa2d5000000000000000000000000000000000000000000000000016345785d8a0000",
        )?.into());
        let mock_timestamp = Some(U64::from(1749040108u64));

        let mock_session_config = SessionSpec {
            signer: address!("0x9BbC92a33F193174bf6Cc09c4b4055500d972479"),
            expires_at: U256::from(1749040108u64),
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(100000000000000000u64),
                period: U256::from(0),
            },
            call_policies: vec![CallSpec {
                target: address!("0x9876543210987654321098765432109876543210"),
                selector: FixedBytes::from([0xa9, 0x05, 0x9c, 0xbb]),
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
        };

        println!(
            "=== Test: encodes session transaction with all parameters ==="
        );
        println!(
            "sessionKeySignedHash: 0x{}",
            hex::encode(&mock_session_key_signed_hash)
        );
        println!("sessionContract: {mock_session_contract}");
        println!("to: {mock_to}");
        if let Some(call_data) = &mock_call_data {
            println!("callData: 0x{}", hex::encode(call_data));
        }
        if let Some(timestamp) = &mock_timestamp {
            println!("timestamp: {timestamp}");
        }

        let args = SessionTransactionEncodedParamsArgs {
            session_key_signed_hash: mock_session_key_signed_hash.into(),
            session_contract: mock_session_contract,
            session_config: mock_session_config,
            to: mock_to,
            call_data: mock_call_data,
            timestamp: mock_timestamp,
        };

        let result = encoded_session_transaction_signature(args)?;

        println!("actual result: 0x{}", hex::encode(&result));
        println!("actual result length: {}", result.len());

        // Expected result from the TypeScript test
        let expected_result = "0x0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000123456789012345678901234567890123456789000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000020a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234560000000000000000000000000000000000000000000000000000000000000380000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000003200000000000000000000000009bbc92a33f193174bf6cc09c4b4055500d9724790000000000000000000000000000000000000000000000000000000068403bec0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000009876543210987654321098765432109876543210a9059cbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000470de4df820000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000b1a2bc2ec50000000000000000000000000000000000000000000000000000000000000001518000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000987654321098765432109876543210987654321000000000000000000000000000000000000000000000000000470de4df820000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000b1a2bc2ec500000000000000000000000000000000000000000000000000000000000000015180000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004f13";

        let expected_bytes = hex::decode(expected_result)?;
        println!("expected_bytes: {expected_bytes:?}");
        println!("result: {result:?}");
        let result_hex = format!("0x{}", hex::encode(&result));

        assert_eq!(
            result_hex, expected_result,
            "Encoded session transaction should match expected result"
        );

        Ok(())
    }
}
