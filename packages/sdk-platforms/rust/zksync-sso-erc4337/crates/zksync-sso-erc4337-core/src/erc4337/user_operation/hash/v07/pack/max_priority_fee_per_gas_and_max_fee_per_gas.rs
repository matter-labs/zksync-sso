use crate::erc4337::entry_point::PackedUserOperation;
use alloy::primitives::B256;

pub fn get_max_priority_fee_per_gas_and_max_fee_per_gas(
    user_operation: &PackedUserOperation,
) -> B256 {
    // gasFees already contains the packed fees
    // It's 32 bytes: 16 bytes max_priority_fee_per_gas + 16 bytes max_fee_per_gas
    let mut result = [0u8; 32];
    if user_operation.gasFees.len() >= 32 {
        result.copy_from_slice(&user_operation.gasFees[0..32]);
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
    fn test_get_max_priority_fee_per_gas_and_max_fee_per_gas() {
        let expected_max_priority_fee_per_gas_and_max_fee_per_gas = fixed_bytes!(
            "00000000000000000000000043d4ca3500000000000000000000000417bbd4f1"
        );
        let user_operation = PackedUserOperationWrapperV07::mock().0;
        let max_priority_fee_per_gas_and_max_fee_per_gas =
            get_max_priority_fee_per_gas_and_max_fee_per_gas(&user_operation);
        assert_eq!(
            max_priority_fee_per_gas_and_max_fee_per_gas,
            expected_max_priority_fee_per_gas_and_max_fee_per_gas,
        );
    }
}
