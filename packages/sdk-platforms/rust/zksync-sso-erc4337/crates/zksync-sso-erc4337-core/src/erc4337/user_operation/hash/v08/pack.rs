pub mod domain;
pub mod hashed_call_data;
pub mod hashed_init_code;
pub mod hashed_paymaster_and_data;
pub mod max_priority_fee_per_gas_and_max_fee_per_gas;
pub mod verificaction_gas_limit_and_call_gas_limit;

use crate::erc4337::entry_point::PackedUserOperation;
use alloy::{
    primitives::{Address, B256, Bytes, U256, keccak256},
    sol_types::{Eip712Domain, SolValue},
};

pub trait CodeReader {
    fn get_code(&self, address: &Address) -> Option<Vec<u8>>;
}

pub fn packed_userop_typehash() -> B256 {
    keccak256(b"PackedUserOperation(address sender,uint256 nonce,bytes initCode,bytes callData,bytes32 accountGasLimits,uint256 preVerificationGas,bytes32 gasFees,bytes paymasterAndData)")
}

pub fn build_domain(entry_point: &Address, chain_id: u64) -> Eip712Domain {
    domain::build_domain(entry_point, chain_id)
}

pub fn struct_hash(
    user_operation: &PackedUserOperation,
    code_reader: &dyn CodeReader,
) -> B256 {
    let typehash = packed_userop_typehash();
    let hash_init_code = hashed_init_code::get_hashed_init_code_with_7702(
        user_operation,
        code_reader,
    );
    let hash_call_data = hashed_call_data::get_hashed_call_data(user_operation);
    let account_gas_limits = verificaction_gas_limit_and_call_gas_limit::get_verificaction_gas_limit_and_call_gas_limit(user_operation);
    let pre_verification_gas: U256 = user_operation.preVerificationGas;
    let gas_fees = max_priority_fee_per_gas_and_max_fee_per_gas::get_max_priority_fee_per_gas_and_max_fee_per_gas(user_operation);
    let hash_paymaster_and_data =
        hashed_paymaster_and_data::get_hashed_paymaster_and_data(
            user_operation,
        );

    let struct_tuple = (
        typehash,
        user_operation.sender,
        user_operation.nonce,
        hash_init_code,
        hash_call_data,
        account_gas_limits,
        pre_verification_gas,
        gas_fees,
        hash_paymaster_and_data,
    );
    let struct_encoded: Bytes = struct_tuple.abi_encode().into();
    keccak256(struct_encoded)
}

pub fn eip712_digest(struct_hash: B256, domain: &Eip712Domain) -> B256 {
    // EIP-712 signing hash: keccak256("\x19\x01" || domainSeparator || structHash)
    let mut data = Vec::new();
    data.extend_from_slice(b"\x19\x01");
    data.extend_from_slice(&domain.separator().0);
    data.extend_from_slice(&struct_hash.0);
    keccak256(data)
}
