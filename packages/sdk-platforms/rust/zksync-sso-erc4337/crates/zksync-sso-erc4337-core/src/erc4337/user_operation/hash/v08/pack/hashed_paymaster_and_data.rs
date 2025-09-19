use crate::erc4337::entry_point::PackedUserOperation;
use alloy::primitives::{B256, Bytes, keccak256};

fn get_data(user_operation: &PackedUserOperation) -> Bytes {
    // For EntryPoint PackedUserOperation, paymasterAndData is already the complete data
    user_operation.paymasterAndData.clone()
}

pub fn get_hashed_paymaster_and_data(
    user_operation: &PackedUserOperation,
) -> B256 {
    let data = get_data(user_operation);
    keccak256(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::user_operation::hash::v08::PackedUserOperationWrapper;

    #[test]
    fn test_get_hashed_paymaster_and_data() {
        let expected_hashed_paymaster_and_data_hex = "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470";
        let user_operation = PackedUserOperationWrapper::mock().0;
        let hashed_paymaster_and_data =
            get_hashed_paymaster_and_data(&user_operation);
        assert_eq!(
            hashed_paymaster_and_data.to_string(),
            expected_hashed_paymaster_and_data_hex
        );
    }
}
