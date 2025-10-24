use crate::erc4337::account::modular_smart_account::session::SessionLib::SessionSpec;
use alloy::{
    primitives::{FixedBytes, keccak256},
    sol_types::SolValue,
};

pub fn hash_session(session_spec: SessionSpec) -> FixedBytes<32> {
    keccak256(session_spec.abi_encode())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::account::modular_smart_account::session::SessionLib::{
        TransferSpec, UsageLimit,
    };
    use alloy::primitives::{U256, Uint, address, fixed_bytes};

    #[test]
    fn test_hash_session() -> eyre::Result<()> {
        let expected_session_hash = fixed_bytes!(
            "0x546d5c65512f40bd0f6924b246258eb29b7411a126f53c06cee2d4941c78c32d"
        );
        let signer_address =
            address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
        let expires_at = Uint::from(2088558400u64);
        let target = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
        let session_spec = SessionSpec {
            signer: signer_address,
            expiresAt: expires_at,
            callPolicies: vec![],
            feeLimit: UsageLimit {
                limitType: 1,
                limit: U256::from(1_000_000_000_000_000_000u64),
                period: Uint::from(0),
            },
            transferPolicies: vec![TransferSpec {
                maxValuePerUse: U256::from(1),
                target,
                valueLimit: UsageLimit {
                    limitType: 0,
                    limit: U256::from(0),
                    period: Uint::from(0),
                },
            }],
        };

        let actual_session_hash = hash_session(session_spec);

        assert_eq!(expected_session_hash, actual_session_hash);

        Ok(())
    }
}
