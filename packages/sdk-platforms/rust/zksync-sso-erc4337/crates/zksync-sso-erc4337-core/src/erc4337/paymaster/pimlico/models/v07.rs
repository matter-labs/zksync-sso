use alloy::{
    primitives::{Address, Bytes, U256},
    rpc::types::erc4337::PackedUserOperation,
};
use serde::{Deserialize, Serialize};

#[derive(
    Default,
    Clone,
    Debug,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationPreSponsorship {
    pub sender: Address,
    pub nonce: U256,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub factory: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub factory_data: Option<Bytes>,
    pub call_data: Bytes,
    pub call_gas_limit: U256,
    pub verification_gas_limit: U256,
    pub pre_verification_gas: U256,
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: U256,
    pub paymaster: Option<Address>,
    pub paymaster_verification_gas_limit: Option<U256>,
    pub paymaster_post_op_gas_limit: Option<U256>,
    pub paymaster_data: Option<Bytes>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub authorization_list: Option<Vec<Authorization>>,
    pub signature: Bytes,
}

impl From<PackedUserOperation> for UserOperationPreSponsorship {
    fn from(user_op: PackedUserOperation) -> Self {
        Self {
            sender: user_op.sender,
            nonce: user_op.nonce,
            factory: user_op.factory,
            factory_data: user_op.factory_data,
            call_data: user_op.call_data,
            call_gas_limit: user_op.call_gas_limit,
            verification_gas_limit: user_op.verification_gas_limit,
            pre_verification_gas: user_op.pre_verification_gas,
            max_fee_per_gas: user_op.max_fee_per_gas,
            max_priority_fee_per_gas: user_op.max_priority_fee_per_gas,
            paymaster: user_op.paymaster,
            paymaster_verification_gas_limit: user_op
                .paymaster_verification_gas_limit,
            paymaster_post_op_gas_limit: user_op.paymaster_post_op_gas_limit,
            paymaster_data: user_op.paymaster_data,
            // authorization_list: user_op.authorization_list,
            signature: user_op.signature,
        }
    }
}

#[derive(
    Default,
    Clone,
    Debug,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct SponsorshipResult {
    pub call_gas_limit: U256,
    pub verification_gas_limit: U256,
    pub pre_verification_gas: U256,
    pub paymaster: Address,
    pub paymaster_verification_gas_limit: U256,
    pub paymaster_post_op_gas_limit: U256,
    pub paymaster_data: Bytes,
}

#[derive(
    Default,
    Clone,
    Debug,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct SponsorshipResponse {
    pub pre_verification_gas: U256,
    pub verification_gas_limit: U256,
    pub call_gas_limit: U256,
    pub paymaster: Address,
    pub paymaster_verification_gas_limit: U256,
    pub paymaster_post_op_gas_limit: U256,
    pub paymaster_data: Bytes,
}

impl SponsorshipResponse {
    pub fn mock() -> Self {
        Self {
            call_gas_limit: U256::from(100000),
            verification_gas_limit: U256::from(100000),
            pre_verification_gas: U256::from(50000),
            paymaster: Address::ZERO,
            paymaster_verification_gas_limit: U256::from(50000),
            paymaster_post_op_gas_limit: U256::from(50000),
            paymaster_data: Bytes::default(),
        }
    }
}
