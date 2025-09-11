use crate::erc4337::entry_point::PackedUserOperation;
use alloy::primitives::{B256, keccak256};

pub fn get_hashed_call_data(user_operation: &PackedUserOperation) -> B256 {
    keccak256(&user_operation.callData)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::user_operation::hash::v08::PackedUserOperationWrapper;

    #[test]
    fn test_get_hashed_call_data() {
        let expected_hashed_call_data_hex = "0x9b96c107f6dc8ddfee294703755263ee85a82c354d68eff28b0781ddcd2c9127";
        let user_operation = PackedUserOperationWrapper::mock().0;
        let hashed_call_data = get_hashed_call_data(&user_operation);
        assert_eq!(hashed_call_data.to_string(), expected_hashed_call_data_hex);
    }
}
