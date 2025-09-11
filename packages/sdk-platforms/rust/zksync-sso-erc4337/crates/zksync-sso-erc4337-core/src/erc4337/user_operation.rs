use crate::erc4337::entry_point::PackedUserOperation;
use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use serde::{Deserialize, Serialize};

pub mod alloy_helpers;
pub mod call;
pub mod estimated;
pub mod hash;
pub mod signature;
pub mod signed;
#[cfg(test)]
pub mod tests;
pub mod utils;
pub mod wrapper_v07;

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

impl From<UserOperationV08> for PackedUserOperation {
    fn from(user_operation: UserOperationV08) -> Self {
        to_packed_user_operation(user_operation)
    }
}

fn to_packed_user_operation(
    user_operation: UserOperationV08,
) -> PackedUserOperation {
    user_operation.to_packed_user_operation()
}

impl UserOperationV08 {
    pub fn to_packed_user_operation(&self) -> PackedUserOperation {
        PackedUserOperation {
            sender: self.sender,
            nonce: self.nonce,
            initCode: get_init_code(self),
            callData: self.call_data.clone(),
            accountGasLimits: get_gas_limits(
                self.verification_gas_limit,
                self.call_gas_limit,
            ),
            preVerificationGas: self.pre_verification_gas,
            gasFees: get_gas_fees(
                self.max_priority_fee_per_gas,
                self.max_fee_per_gas,
            ),
            paymasterAndData: get_paymaster_and_data(self),
            signature: self.signature.clone(),
        }
    }
}

fn get_gas_limits(
    verification_gas_limit: U256,
    call_gas_limit: U256,
) -> FixedBytes<32> {
    utils::pack_u256_pair(verification_gas_limit, call_gas_limit)
}

fn get_init_code(user_operation: &UserOperationV08) -> Bytes {
    match (user_operation.factory, &user_operation.factory_data) {
        (Some(factory), Some(factory_data)) => {
            utils::concat_factory_data(factory, factory_data)
        }
        _ => Bytes::default(),
    }
}

fn get_gas_fees(
    max_priority_fee_per_gas: U256,
    max_fee_per_gas: U256,
) -> FixedBytes<32> {
    utils::pack_u256_pair(max_priority_fee_per_gas, max_fee_per_gas)
}

fn get_paymaster_and_data(user_operation: &UserOperationV08) -> Bytes {
    match user_operation.paymaster {
        Some(paymaster) => {
            let verification_gas_limit = user_operation
                .paymaster_verification_gas_limit
                .unwrap_or_default();
            let post_op_gas_limit =
                user_operation.paymaster_post_op_gas_limit.unwrap_or_default();
            let paymaster_data = user_operation
                .paymaster_data
                .as_ref()
                .cloned()
                .unwrap_or_default();
            utils::concat_paymaster_data(
                paymaster,
                verification_gas_limit,
                post_op_gas_limit,
                &paymaster_data,
            )
        }
        None => Bytes::default(),
    }
}
