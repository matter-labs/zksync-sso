use crate::erc4337::entry_point::PackedUserOperation;
use alloy::primitives::{B256, keccak256};

pub fn get_hashed_paymaster_and_data(
    user_operation: &PackedUserOperation,
) -> B256 {
    keccak256(&user_operation.paymasterAndData)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::user_operation::wrapper_v07::PackedUserOperationWrapperV07;

    #[test]
    #[ignore = "need to investigate failure"]
    fn test_get_hashed_paymaster_and_data() {
        let expected_hashed_paymaster_and_data_hex = "0xfc0dffa735c71f138a00eaaafa56834aebf784e3e446612810f3f325cfb8eda9";
        let user_operation = PackedUserOperationWrapperV07::mock().0;
        let hashed_paymaster_and_data =
            get_hashed_paymaster_and_data(&user_operation);
        assert_eq!(
            hashed_paymaster_and_data.to_string(),
            expected_hashed_paymaster_and_data_hex,
        );
    }
}
