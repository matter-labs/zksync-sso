use crate::erc4337::{
    entry_point::PackedUserOperation,
    user_operation::hash::{user_operation_hash::UserOperationHash, v07},
};
use alloy::primitives::{Address, Bytes, FixedBytes, U256};

pub struct PackedUserOperationWrapperV07(pub PackedUserOperation);

impl Default for PackedUserOperationWrapperV07 {
    fn default() -> Self {
        let inner = PackedUserOperation {
            sender: Address::ZERO,
            nonce: U256::ZERO,
            initCode: Bytes::default(),
            callData: Bytes::default(),
            accountGasLimits: FixedBytes::<32>::ZERO, // 16 bytes verification + 16 bytes call
            preVerificationGas: U256::ZERO,
            gasFees: FixedBytes::<32>::ZERO, // 16 bytes priority + 16 bytes max fee
            paymasterAndData: Bytes::default(),
            signature: Bytes::default(),
        };
        Self(inner)
    }
}

impl From<PackedUserOperation> for PackedUserOperationWrapperV07 {
    fn from(value: PackedUserOperation) -> Self {
        Self(value)
    }
}

impl PackedUserOperationWrapperV07 {
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
    PackedUserOperationWrapperV07::default().0
}
