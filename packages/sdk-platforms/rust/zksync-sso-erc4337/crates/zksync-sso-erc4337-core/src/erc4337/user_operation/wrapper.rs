use crate::erc4337::user_operation::hash::{
    user_operation_hash::UserOperationHash, v07,
};
use alloy::{
    primitives::{Address, Bytes, U256},
    rpc::types::erc4337::PackedUserOperation,
};

pub struct PackedUserOperationWrapper(pub PackedUserOperation);

impl Default for PackedUserOperationWrapper {
    fn default() -> Self {
        let inner = PackedUserOperation {
            sender: Address::ZERO,
            nonce: U256::ZERO,
            factory: None,
            factory_data: None,
            call_data: Bytes::default(),
            call_gas_limit: U256::ZERO,
            verification_gas_limit: U256::ZERO,
            pre_verification_gas: U256::ZERO,
            max_fee_per_gas: U256::ZERO,
            max_priority_fee_per_gas: U256::ZERO,
            paymaster: None,
            paymaster_verification_gas_limit: None,
            paymaster_post_op_gas_limit: None,
            paymaster_data: None,
            signature: Bytes::default(),
        };
        Self(inner)
    }
}

impl From<PackedUserOperation> for PackedUserOperationWrapper {
    fn from(value: PackedUserOperation) -> Self {
        Self(value)
    }
}

impl PackedUserOperationWrapper {
    /// Calculates the hash of the user operation
    pub fn hash(
        &self,
        entry_point: &Address,
        chain_id: u64,
    ) -> UserOperationHash {
        v07::get_user_operation_hash(&self.0, entry_point, chain_id)
    }
}

pub fn create_user_operation() -> PackedUserOperation {
    PackedUserOperationWrapper::default().0
}
