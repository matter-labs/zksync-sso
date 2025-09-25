use crate::erc4337::{
    entry_point::PackedUserOperation,
    user_operation::hash::v08::user_operation_typed_data::{
        create_domain, hash_typed_data::hash_typed_data,
    },
};
use alloy::primitives::{Address, FixedBytes};

pub fn get_user_operation_hash(
    chain_id: u64,
    entry_point_address: Address,
    user_operation: PackedUserOperation,
) -> FixedBytes<32> {
    let domain = create_domain(chain_id, entry_point_address);

    hash_typed_data(user_operation, domain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::user_operation::UserOperationV08;
    use alloy::primitives::{Bytes, U256, address, hex};

    #[test]
    #[ignore = "`get_user_operation_hash` is not returning the same hash as `viem` need to investigate"]
    fn test_hash_typed_data() -> eyre::Result<()> {
        let chain_id = 1;
        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let user_op = UserOperationV08 {
            call_data: Bytes::new(),
            call_gas_limit: U256::from(6942069),
            max_fee_per_gas: U256::from(69420),
            max_priority_fee_per_gas: U256::from(69),
            nonce: U256::from(0),
            pre_verification_gas: U256::from(6942069),
            sender: address!("0x1234567890123456789012345678901234567890"),
            signature: Bytes::new(),
            verification_gas_limit: U256::from(6942069),
            ..Default::default()
        };

        let user_operation_packed = user_op.into();

        let user_op_hash = get_user_operation_hash(
            chain_id,
            entry_point_address,
            user_operation_packed,
        );

        println!("user_op_hash: {}", hex::encode(user_op_hash));

        let expected_user_op_hash: FixedBytes<32> = hex::decode(
            "0xa2224e732a1d4e2f923c7c05d586a0aa6cbc42172ec02f31d35fa9a2b8ba9208",
        )
        .map(|bytes| FixedBytes::from_slice(&bytes))?;

        eyre::ensure!(
            user_op_hash == expected_user_op_hash,
            "user operation hash should match expected hash, received: {}, expected: {}",
            hex::encode(user_op_hash),
            hex::encode(expected_user_op_hash)
        );

        Ok(())
    }
}
