use alloy::{
    primitives::{B256, Bytes, keccak256},
    rpc::types::erc4337::PackedUserOperation,
};

fn get_data(user_operation: &PackedUserOperation) -> Bytes {
    let Some(paymaster) = user_operation.paymaster else {
        return Bytes::new();
    };

    let address = paymaster.into_iter();

    let paymaster_verification_gas_limit = user_operation
        .paymaster_verification_gas_limit
        .unwrap_or_default()
        .to_be_bytes_vec()
        .into_iter()
        .skip(16);

    let paymaster_post_op_gas_limit = user_operation
        .paymaster_post_op_gas_limit
        .unwrap_or_default()
        .to_be_bytes_vec()
        .into_iter()
        .skip(16);

    let paymaster_data = user_operation
        .paymaster_data
        .as_ref()
        .unwrap_or_default()
        .iter()
        .copied();

    address
        .chain(paymaster_verification_gas_limit)
        .chain(paymaster_post_op_gas_limit)
        .chain(paymaster_data)
        .collect()
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
    use crate::erc4337::user_operation::wrapper::PackedUserOperationWrapper;

    #[test]
    fn test_get_hashed_paymaster_and_data() {
        let expected_hashed_paymaster_and_data_hex = "0xfc0dffa735c71f138a00eaaafa56834aebf784e3e446612810f3f325cfb8eda9";
        let user_operation = PackedUserOperationWrapper::mock().0;
        let hashed_paymaster_and_data =
            get_hashed_paymaster_and_data(&user_operation);
        assert_eq!(
            hashed_paymaster_and_data.to_string(),
            expected_hashed_paymaster_and_data_hex,
        );
    }
}
