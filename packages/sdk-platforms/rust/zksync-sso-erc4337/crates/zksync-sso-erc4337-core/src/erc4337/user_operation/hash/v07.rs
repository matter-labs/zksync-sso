use crate::erc4337::{
    entry_point::PackedUserOperation,
    user_operation::hash::{
        user_operation_hash::UserOperationHash, v07::pack::pack_user_operation,
    },
};
use alloy::{
    primitives::{Address, B256, Bytes, U256, keccak256},
    sol_types::SolValue,
};

pub mod pack;

pub fn get_user_operation_hash(
    user_operation: &PackedUserOperation,
    entry_point: &Address,
    chain_id: u64,
) -> UserOperationHash {
    let packed_user_operation = {
        let packed = pack_user_operation(user_operation);
        println!("packed: {:?}", packed);
        keccak256(packed)
    };
    println!("packed_user_operation: {:?}", packed_user_operation);

    let chain_id = U256::from(chain_id);

    let values = (packed_user_operation, entry_point, chain_id);
    let abi_encoded = values.abi_encode();
    let abi_encoded_packed = values.abi_encode_packed();
    println!("abi_encoded: {:?}", abi_encoded.clone());
    println!("abi_encoded_packed: {:?}", abi_encoded_packed.clone());
    assert_eq!(values.sol_name(), "(bytes32,address,uint256)");

    let encoded: Bytes = abi_encoded.into();
    let hash_bytes = keccak256(encoded);
    let hash = B256::from_slice(hash_bytes.as_slice());
    UserOperationHash(hash)
}
