use crate::erc4337::user_operation::hash::user_operation_hash::UserOperationHash;
use alloy::{
    primitives::{Bytes, FixedBytes},
    signers::{SignerSync, local::PrivateKeySigner},
};

pub fn sign_user_operation_hash(
    user_operation_hash: &UserOperationHash,
    signer: &PrivateKeySigner,
) -> eyre::Result<Bytes> {
    let hash_fixed: FixedBytes<32> =
        FixedBytes::from_slice(user_operation_hash.as_fixed_bytes());
    let signature = signer.sign_hash_sync(&hash_fixed)?;
    let signature_bytes = signature.as_bytes();
    let signature_fixed: FixedBytes<65> =
        FixedBytes::from_slice(signature_bytes.as_slice());
    Ok(signature_fixed.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{
        TEST_EXPECTED_SIGNATURE, TEST_PRIVATE_KEY, TEST_USER_OP_HASH,
    };
    use std::str::FromStr;

    #[test]
    fn test_sign_user_operation_hash_matches_viem_example() -> eyre::Result<()>
    {
        let user_operation_hash =
            UserOperationHash::from_str(TEST_USER_OP_HASH)
                .expect("invalid hash");
        let signer = PrivateKeySigner::from_str(TEST_PRIVATE_KEY)
            .map_err(|_| eyre::eyre!("invalid private key"))?;

        let signature = sign_user_operation_hash(&user_operation_hash, &signer)
            .expect("failed to sign");

        let expected = Bytes::from_str(TEST_EXPECTED_SIGNATURE)
            .expect("invalid expected signature hex");

        assert_eq!(signature, expected);

        Ok(())
    }
}
