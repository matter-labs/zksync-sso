use crate::erc4337::entry_point::contract::{
    EntryPoint::getUserOpHashCall, PackedUserOperation,
};
use alloy::{primitives::Bytes, sol_types::SolCall};

pub fn get_user_op_hash_call_data(
    user_operation: PackedUserOperation,
) -> eyre::Result<Bytes> {
    let call = getUserOpHashCall { userOp: user_operation };
    Ok(call.abi_encode().into())
}
