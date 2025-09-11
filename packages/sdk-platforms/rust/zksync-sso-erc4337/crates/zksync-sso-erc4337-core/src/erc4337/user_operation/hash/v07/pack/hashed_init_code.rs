use crate::erc4337::entry_point::PackedUserOperation;
use alloy::primitives::{B256, keccak256};

pub fn get_hashed_init_code(user_operation: &PackedUserOperation) -> B256 {
    keccak256(&user_operation.initCode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::user_operation::wrapper_v07::PackedUserOperationWrapperV07;

    #[test]
    #[ignore = "need to investigate failure"]
    fn test_get_hashed_init_code() {
        let expected_hashed_init_code_hex = "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470";
        let user_operation = PackedUserOperationWrapperV07::mock().0;
        let hashed_init_code = get_hashed_init_code(&user_operation);
        assert_eq!(hashed_init_code.to_string(), expected_hashed_init_code_hex);
    }
}
