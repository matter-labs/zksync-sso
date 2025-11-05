use crate::erc4337::user_operation::utils::concat_paymaster_data;
use alloy::primitives::{Address, Bytes, U256};

#[derive(Debug, Clone)]
pub struct PaymasterParams {
    pub address: Address,
    pub data: Bytes,
    pub verification_gas_limit: Option<U256>,
    pub post_op_gas_limit: Option<U256>,
}

impl PaymasterParams {
    pub fn default_paymaster(address: Address) -> Self {
        Self {
            address,
            data: Bytes::default(),
            verification_gas_limit: None,
            post_op_gas_limit: None,
        }
    }
}

pub fn build_paymaster_and_data(
    paymaster: Option<Address>,
    verification_gas_limit: Option<U256>,
    post_op_gas_limit: Option<U256>,
    paymaster_data: Option<&Bytes>,
) -> Bytes {
    let Some(paymaster_address) = paymaster else {
        return Bytes::default();
    };
    let verification = verification_gas_limit.unwrap_or_default();
    let post_op = post_op_gas_limit.unwrap_or_default();
    let data = paymaster_data.cloned().unwrap_or_default();
    concat_paymaster_data(paymaster_address, verification, post_op, &data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Bytes, U256, address};

    #[test]
    fn test_build_paymaster_and_data_with_values() {
        let paymaster = address!("0x1111111111111111111111111111111111111111");
        let verification = U256::from(0x1000);
        let post_op = U256::from(0x2000);
        let data = Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]);

        let result = build_paymaster_and_data(
            Some(paymaster),
            Some(verification),
            Some(post_op),
            Some(&data),
        );

        let expected =
            concat_paymaster_data(paymaster, verification, post_op, &data);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_paymaster_and_data_without_paymaster() {
        let data = Bytes::from(vec![0xca, 0xfe]);
        let result = build_paymaster_and_data(
            None,
            Some(U256::from(0x1)),
            Some(U256::from(0x2)),
            Some(&data),
        );
        assert_eq!(result, Bytes::default());
    }
}
