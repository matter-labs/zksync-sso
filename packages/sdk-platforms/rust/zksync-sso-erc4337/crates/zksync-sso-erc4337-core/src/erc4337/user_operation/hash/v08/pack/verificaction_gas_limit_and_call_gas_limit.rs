use crate::erc4337::{
    entry_point::PackedUserOperation,
    user_operation::hash::combine::combine_and_trim_first_16_bytes,
};
use alloy::primitives::{B256, U256};

pub fn get_verificaction_gas_limit_and_call_gas_limit(
    user_operation: &PackedUserOperation,
) -> B256 {
    // For EntryPoint PackedUserOperation, we need to extract from accountGasLimits
    // accountGasLimits is packed as: verificationGasLimit (128 bits) + callGasLimit (128 bits)
    let account_gas_limits = user_operation.accountGasLimits;
    let verification_gas_limit = u128::from_be_bytes([
        account_gas_limits[16],
        account_gas_limits[17],
        account_gas_limits[18],
        account_gas_limits[19],
        account_gas_limits[20],
        account_gas_limits[21],
        account_gas_limits[22],
        account_gas_limits[23],
        account_gas_limits[24],
        account_gas_limits[25],
        account_gas_limits[26],
        account_gas_limits[27],
        account_gas_limits[28],
        account_gas_limits[29],
        account_gas_limits[30],
        account_gas_limits[31],
    ]);
    let call_gas_limit = u128::from_be_bytes([
        account_gas_limits[0],
        account_gas_limits[1],
        account_gas_limits[2],
        account_gas_limits[3],
        account_gas_limits[4],
        account_gas_limits[5],
        account_gas_limits[6],
        account_gas_limits[7],
        account_gas_limits[8],
        account_gas_limits[9],
        account_gas_limits[10],
        account_gas_limits[11],
        account_gas_limits[12],
        account_gas_limits[13],
        account_gas_limits[14],
        account_gas_limits[15],
    ]);

    combine_and_trim_first_16_bytes(
        U256::from(verification_gas_limit),
        U256::from(call_gas_limit),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::user_operation::hash::v08::PackedUserOperationWrapper;

    #[test]
    fn test_get_verificaction_gas_limit_and_call_gas_limit() {
        let expected_verification_gas_limit_and_call_gas_limit = "0x0000000000000000000000000000e7a800000000000000000000000000003fc3";
        let user_operation = PackedUserOperationWrapper::mock().0;
        let verification_gas_limit_and_call_gas_limit =
            get_verificaction_gas_limit_and_call_gas_limit(&user_operation);
        assert_eq!(
            verification_gas_limit_and_call_gas_limit.to_string(),
            expected_verification_gas_limit_and_call_gas_limit
        );
    }
}
