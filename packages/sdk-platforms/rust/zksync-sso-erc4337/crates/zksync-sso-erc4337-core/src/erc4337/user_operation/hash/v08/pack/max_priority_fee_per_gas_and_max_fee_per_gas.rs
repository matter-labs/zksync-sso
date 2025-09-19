use crate::erc4337::{
    entry_point::PackedUserOperation,
    user_operation::hash::combine::combine_and_trim_first_16_bytes,
};
use alloy::primitives::{B256, U256};

pub fn get_max_priority_fee_per_gas_and_max_fee_per_gas(
    user_operation: &PackedUserOperation,
) -> B256 {
    // For EntryPoint PackedUserOperation, we need to extract from gasFees
    // gasFees is packed as: maxPriorityFeePerGas (128 bits) + maxFeePerGas (128 bits)
    let gas_fees = user_operation.gasFees;
    let max_priority_fee_per_gas = u128::from_be_bytes([
        gas_fees[16],
        gas_fees[17],
        gas_fees[18],
        gas_fees[19],
        gas_fees[20],
        gas_fees[21],
        gas_fees[22],
        gas_fees[23],
        gas_fees[24],
        gas_fees[25],
        gas_fees[26],
        gas_fees[27],
        gas_fees[28],
        gas_fees[29],
        gas_fees[30],
        gas_fees[31],
    ]);
    let max_fee_per_gas = u128::from_be_bytes([
        gas_fees[0],
        gas_fees[1],
        gas_fees[2],
        gas_fees[3],
        gas_fees[4],
        gas_fees[5],
        gas_fees[6],
        gas_fees[7],
        gas_fees[8],
        gas_fees[9],
        gas_fees[10],
        gas_fees[11],
        gas_fees[12],
        gas_fees[13],
        gas_fees[14],
        gas_fees[15],
    ]);

    combine_and_trim_first_16_bytes(
        U256::from(max_priority_fee_per_gas),
        U256::from(max_fee_per_gas),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::user_operation::hash::v08::PackedUserOperationWrapper;

    #[test]
    fn test_get_max_priority_fee_per_gas_and_max_fee_per_gas() {
        let expected_max_priority_fee_per_gas_and_max_fee_per_gas = "0x000000000000000000000000773594000000000000000000000000008585115a";
        let user_operation = PackedUserOperationWrapper::mock().0;
        let max_priority_fee_per_gas_and_max_fee_per_gas =
            get_max_priority_fee_per_gas_and_max_fee_per_gas(&user_operation);
        assert_eq!(
            max_priority_fee_per_gas_and_max_fee_per_gas.to_string(),
            expected_max_priority_fee_per_gas_and_max_fee_per_gas
        );
    }
}
