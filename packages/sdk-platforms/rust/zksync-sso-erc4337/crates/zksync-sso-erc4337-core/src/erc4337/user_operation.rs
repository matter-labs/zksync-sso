use crate::erc4337::entry_point::PackedUserOperation;
use alloy::primitives::{Address, Bytes, U256};
use serde::{Deserialize, Serialize};

pub mod alloy_helpers;
pub mod hash;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedAuthorization {
    // TODO: Implement the SignedAuthorization struct
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct UserOperationV08 {
    // Authorization data
    pub authorization: Option<SignedAuthorization>,

    // The data to pass to the `sender` during the main execution call.
    pub call_data: Bytes,

    // The amount of gas to allocate the main execution call
    pub call_gas_limit: U256,

    // Account factory. Only for new accounts
    pub factory: Option<Address>,

    // Data for account factory
    pub factory_data: Option<Bytes>,

    // Maximum fee per gas
    pub max_fee_per_gas: U256,

    // Maximum priority fee per gas
    pub max_priority_fee_per_gas: U256,

    // Anti-replay parameter
    pub nonce: U256,

    // Address of paymaster contract
    pub paymaster: Option<Address>,

    // Data for paymaster
    pub paymaster_data: Option<Bytes>,

    // The amount of gas to allocate for the paymaster post-operation code
    pub paymaster_post_op_gas_limit: Option<U256>,

    // The amount of gas to allocate for the paymaster validation code
    pub paymaster_verification_gas_limit: Option<U256>,

    // Extra gas to pay the Bundler
    pub pre_verification_gas: U256,

    // The account making the operation
    pub sender: Address,

    // Data passed into the account to verify authorization
    pub signature: Bytes,

    // The amount of gas to allocate for the verification step
    pub verification_gas_limit: U256,
}
