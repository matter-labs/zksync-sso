use crate::erc4337::user_operation::hash::combine::combine_and_trim_first_16_bytes;
use alloy::{primitives::B256, rpc::types::erc4337::PackedUserOperation};

pub fn get_max_priority_fee_per_gas_and_max_fee_per_gas(
    user_operation: &PackedUserOperation,
) -> B256 {
    combine_and_trim_first_16_bytes(
        user_operation.max_priority_fee_per_gas,
        user_operation.max_fee_per_gas,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::user_operation::wrapper::PackedUserOperationWrapper;
    use alloy::primitives::fixed_bytes;

    #[test]
    fn test_get_max_priority_fee_per_gas_and_max_fee_per_gas() {
        let expected_max_priority_fee_per_gas_and_max_fee_per_gas = fixed_bytes!(
            "00000000000000000000000043d4ca3500000000000000000000000417bbd4f1"
        );
        let user_operation = PackedUserOperationWrapper::mock().0;
        let max_priority_fee_per_gas_and_max_fee_per_gas =
            get_max_priority_fee_per_gas_and_max_fee_per_gas(&user_operation);
        assert_eq!(
            max_priority_fee_per_gas_and_max_fee_per_gas,
            expected_max_priority_fee_per_gas_and_max_fee_per_gas,
        );
    }
}
