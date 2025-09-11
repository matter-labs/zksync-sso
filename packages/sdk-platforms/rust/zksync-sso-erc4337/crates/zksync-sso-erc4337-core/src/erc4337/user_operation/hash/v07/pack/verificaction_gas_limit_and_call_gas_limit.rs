use crate::erc4337::user_operation::hash::combine::combine_and_trim_first_16_bytes;
use alloy::{primitives::B256, rpc::types::erc4337::PackedUserOperation};

pub fn get_verificaction_gas_limit_and_call_gas_limit(
    user_operation: &PackedUserOperation,
) -> B256 {
    combine_and_trim_first_16_bytes(
        user_operation.verification_gas_limit,
        user_operation.call_gas_limit,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::user_operation::wrapper::PackedUserOperationWrapper;
    use alloy::primitives::fixed_bytes;

    #[test]
    fn test_get_verificaction_gas_limit_and_call_gas_limit() {
        let expected_verification_gas_limit_and_call_gas_limit = fixed_bytes!(
            "00000000000000000000000000010b2500000000000000000000000000013880"
        );
        let user_operation = PackedUserOperationWrapper::mock().0;
        let verification_gas_limit_and_call_gas_limit =
            get_verificaction_gas_limit_and_call_gas_limit(&user_operation);
        assert_eq!(
            verification_gas_limit_and_call_gas_limit,
            expected_verification_gas_limit_and_call_gas_limit,
        );
    }
}
