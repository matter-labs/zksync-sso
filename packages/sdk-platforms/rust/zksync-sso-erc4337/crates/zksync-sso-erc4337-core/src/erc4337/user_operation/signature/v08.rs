use crate::erc4337::{
    entry_point::PackedUserOperation,
    user_operation::{
        hash::v08::get_user_operation_hash,
        signature::v08::sign_hash::sign_user_operation_hash,
    },
};
use alloy::{primitives::Address, signers::local::PrivateKeySigner};

pub mod sign_hash;

pub fn sign_user_operation(
    user_operation: &PackedUserOperation,
    signer: &PrivateKeySigner,
    chain_id: u64,
    entry_point_address: Address,
) -> eyre::Result<PackedUserOperation> {
    let hash =
        get_user_operation_hash(user_operation, &entry_point_address, chain_id);

    let signature = sign_user_operation_hash(&hash, signer)?;

    let mut user_operation = user_operation.clone();
    user_operation.signature = signature;

    Ok(user_operation)
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{
        TEST_EXPECTED_SIGNATURE, TEST_PRIVATE_KEY, load_userop_from_fixture,
    };
    use alloy::{primitives::Bytes, signers::local::PrivateKeySigner};
    use std::str::FromStr;

    #[test]
    fn test_sign_user_operation_matches_viem_example() {
        let (user_operation, entry_point, chain_id) = load_userop_from_fixture(
            "../../../../../erc4337-contracts/test/integration/erc4337-userop.json",
        );

        let signer = PrivateKeySigner::from_str(TEST_PRIVATE_KEY).unwrap();

        let signed = super::sign_user_operation(
            &user_operation,
            &signer,
            chain_id,
            entry_point,
        )
        .expect("failed to sign user operation");

        let expected_signature =
            Bytes::from_str(TEST_EXPECTED_SIGNATURE).unwrap();

        assert_eq!(signed.signature, expected_signature);
    }
}
