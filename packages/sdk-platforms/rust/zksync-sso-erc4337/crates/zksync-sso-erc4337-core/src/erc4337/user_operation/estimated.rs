use crate::erc4337::entry_point::contract::PackedUserOperation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserOperationEstimated(PackedUserOperation);

impl From<UserOperationEstimated> for PackedUserOperation {
    fn from(val: UserOperationEstimated) -> Self {
        val.0
    }
}

#[derive(Debug, Clone)]
pub struct SignedUserOperation(PackedUserOperation);

impl From<SignedUserOperation> for PackedUserOperation {
    fn from(val: SignedUserOperation) -> Self {
        val.0
    }
}
