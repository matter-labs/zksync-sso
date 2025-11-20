use crate::{
    contracts::SessionLib::SessionSpec as SessionLibSessionSpec,
    utils::session::{
        period_ids_for_transaction::get_period_ids_for_transaction,
        session_lib::session_spec::SessionSpec,
    },
};
use alloy::{
    primitives::{Address, Bytes, FixedBytes, U64},
    sol,
    sol_types::SolType,
};
use eyre::Result;
use log::debug;

sol! {
    struct SessionParams {
        SessionLibSessionSpec memory sessionSpec;
        uint64[] periodIds;
    }
}

pub(crate) fn encode_session_tx(
    session_config: SessionSpec,
    to: Address,
    call_data: Option<Bytes>,
    timestamp: Option<U64>,
) -> Result<Bytes> {
    debug!("XDB - encode_session_tx - session_config: {session_config:?}");

    let session_spec: SessionSpec = session_config.clone();

    let selector = call_data.map(extract_selector_from_call_data);

    let period_ids = get_period_ids_for_transaction(
        &session_config.into(),
        to,
        selector,
        timestamp,
    )?
    .into_iter()
    .map(u64::try_from)
    .map(|id| id.unwrap_or(u64::MAX))
    .collect();

    let params = SessionParams {
        sessionSpec: session_spec.into(),
        periodIds: period_ids,
    };

    let params_bytes = <SessionParams as SolType>::abi_encode_params(&params);

    Ok(params_bytes.into())
}

fn extract_selector_from_call_data(call_data: Bytes) -> FixedBytes<4> {
    debug!("extract_selector_from_call_data");
    debug!("extract_selector_from_call_data - call_data: {call_data:?}");
    // Extract the first 4 bytes (function selector) from call data
    // and return as FixedBytes<32> with the selector in the first 4 bytes
    let mut selector_array = [0u8; 4];
    selector_array[..4].copy_from_slice(&call_data[..4]);
    let selector = FixedBytes::<4>::from(selector_array);
    debug!("extract_selector_from_call_data - selector: {selector:?}");
    selector
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::SessionLib::{
        CallSpec, SessionSpec as SessionLibSessionSpec, TransferSpec, UsageLimit,
    };
    use alloy::primitives::{FixedBytes, U256, address, fixed_bytes, hex};

    #[test]
    fn test_encodes_session_transaction_with_call_data() -> eyre::Result<()> {
        let expected_encoded: Bytes = hex::decode(
            "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000003200000000000000000000000009bbc92a33f193174bf6cc09c4b4055500d9724790000000000000000000000000000000000000000000000000000000068403bec0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000001111111111111111111111111111111111111111a9059cbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000038d7ea4c680000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000002386f26fc10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000222222222222222222222222222222222222222200000000000000000000000000000000000000000000000000470de4df820000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        )?.into();
        let session_config: SessionLibSessionSpec = SessionLibSessionSpec {
            signer: address!("0x9BbC92a33F193174bf6Cc09c4b4055500d972479"),
            expiresAt: U256::from(1749040108u64),
            feeLimit: UsageLimit {
                limitType: 1, // Lifetime
                limit: U256::from(100000000000000000u64),
                period: U256::from(0),
            },
            callPolicies: vec![CallSpec {
                target: address!("0x1111111111111111111111111111111111111111"),
                selector: fixed_bytes!("0xa9059cbb"),
                maxValuePerUse: U256::from(1000000000000000u64),
                valueLimit: UsageLimit {
                    limitType: 1, // Lifetime
                    limit: U256::from(10000000000000000u64),
                    period: U256::from(0),
                },
                constraints: vec![],
            }],
            transferPolicies: vec![TransferSpec {
                target: address!("0x2222222222222222222222222222222222222222"),
                maxValuePerUse: U256::from(20000000000000000u64),
                valueLimit: UsageLimit {
                    limitType: 0, // Allowance
                    limit: U256::from(0),
                    period: U256::from(0),
                },
            }],
        };

        let to = address!("0x1111111111111111111111111111111111111111");
        let call_data = Some(hex::decode(
            "a9059cbb000000000000000000000000742d35cc6481c2962f73827c58e76db6d23aa2d5000000000000000000000000000000000000000000000000016345785d8a0000",
        )?.into());
        let timestamp = Some(U64::from(1749040108u64));

        let encoded =
            encode_session_tx(session_config.into(), to, call_data, timestamp)?;

        eyre::ensure!(
            encoded == expected_encoded,
            "Encoded session transaction 0x{} does not match expected encoded 0x{}",
            hex::encode(&encoded),
            hex::encode(&expected_encoded)
        );

        Ok(())
    }

    #[test]
    fn test_extract_selector_from_call_data() -> eyre::Result<()> {
        // Create expected selector as FixedBytes<32> with selector in first 4 bytes
        let expected_selector: FixedBytes<4> = fixed_bytes!("a9059cbb");

        // Convert to Bytes
        let call_data: Bytes = hex::decode(
            "a9059cbb000000000000000000000000742d35cc6481c2962f73827c58e76db6d23aa2d5000000000000000000000000000000000000000000000000016345785d8a0000",
        )?.into();

        let selector = extract_selector_from_call_data(call_data);
        eyre::ensure!(
            selector == expected_selector,
            "Selector {:?} does not match expected selector {:?}",
            selector,
            expected_selector
        );

        Ok(())
    }
}
