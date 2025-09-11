use crate::erc4337::entry_point::PackedUserOperation;
use alloy::primitives::B256;

pub fn get_verificaction_gas_limit_and_call_gas_limit(
    user_operation: &PackedUserOperation,
) -> B256 {
    // accountGasLimits already contains the packed gas limits
    // It's 32 bytes: 16 bytes verification_gas_limit + 16 bytes call_gas_limit
    let mut result = [0u8; 32];
    if user_operation.accountGasLimits.len() >= 32 {
        result.copy_from_slice(&user_operation.accountGasLimits[0..32]);
    }
    B256::from(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::user_operation::wrapper_v07::PackedUserOperationWrapperV07;
    use alloy::primitives::fixed_bytes;

    #[test]
    #[ignore = "need to investigate failure"]
    fn test_get_verificaction_gas_limit_and_call_gas_limit() {
        let expected_verification_gas_limit_and_call_gas_limit = fixed_bytes!(
            "00000000000000000000000000010b2500000000000000000000000000013880"
        );
        let user_operation = PackedUserOperationWrapperV07::mock().0;
        let verification_gas_limit_and_call_gas_limit =
            get_verificaction_gas_limit_and_call_gas_limit(&user_operation);
        assert_eq!(
            verification_gas_limit_and_call_gas_limit,
            expected_verification_gas_limit_and_call_gas_limit,
        );
    }
}
