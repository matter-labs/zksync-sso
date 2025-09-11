use crate::erc4337::entry_point::PackedUserOperation;
use alloy::sol_types::SolValue;

pub mod hashed_call_data;
pub mod hashed_init_code;
pub mod hashed_paymaster_and_data;
pub mod max_priority_fee_per_gas_and_max_fee_per_gas;
pub mod verificaction_gas_limit_and_call_gas_limit;

pub fn pack_user_operation(user_operation: &PackedUserOperation) -> Vec<u8> {
    let hashed_init_code =
        hashed_init_code::get_hashed_init_code(user_operation);

    let hashed_call_data =
        hashed_call_data::get_hashed_call_data(user_operation);

    let hashed_paymaster_and_data =
        hashed_paymaster_and_data::get_hashed_paymaster_and_data(
            user_operation,
        );

    let verificaction_gas_limit_and_call_gas_limit_item =
        verificaction_gas_limit_and_call_gas_limit::get_verificaction_gas_limit_and_call_gas_limit(user_operation);

    let max_priority_fee_per_gas_and_max_fee_per_gas_item =
        max_priority_fee_per_gas_and_max_fee_per_gas::get_max_priority_fee_per_gas_and_max_fee_per_gas(user_operation);

    let items = (
        user_operation.sender,
        user_operation.nonce,
        hashed_init_code,
        hashed_call_data,
        verificaction_gas_limit_and_call_gas_limit_item,
        user_operation.preVerificationGas,
        max_priority_fee_per_gas_and_max_fee_per_gas_item,
        hashed_paymaster_and_data,
    );

    items.abi_encode()
}
