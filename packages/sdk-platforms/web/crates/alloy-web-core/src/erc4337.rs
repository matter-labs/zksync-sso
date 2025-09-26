pub mod account;
pub mod contracts;
pub mod infrastructure;

use crate::utils::create_error;
use alloy::primitives::{Address, Bytes, FixedBytes, U128, U256};
use alloy::rpc::types::eth::erc4337::UserOperation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackedUserOperation {
    pub sender: Address,
    pub nonce: U256,
    pub init_code: Bytes,
    pub call_data: Bytes,
    pub account_gas_limits: FixedBytes<32>,
    pub pre_verification_gas: U256,
    pub gas_fees: FixedBytes<32>,
    pub paymaster_and_data: Bytes,
    pub signature: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum SendUserOperation {
    #[serde(rename = "v0.6")]
    EntryPointV06 { user_op: UserOperation },
    #[serde(rename = "v0.7")]
    EntryPointV07 { packed_user_op: PackedUserOperation },
}

#[derive(Deserialize, Debug)]
pub struct UserOpParams {
    pub sender: String,
    pub nonce: Option<String>,
    pub init_code: Option<String>,
    pub call_data: Option<String>,
    pub call_gas_limit: Option<String>,
    pub verification_gas_limit: Option<String>,
    pub pre_verification_gas: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub paymaster_and_data: Option<String>,
    pub signature: Option<String>,
    pub entry_point_version: Option<String>,
}

#[derive(Serialize)]
pub struct UserOperationResult {
    pub sender: String,
    pub nonce: String,
    pub init_code: String,
    pub call_data: String,
    pub call_gas_limit: String,
    pub verification_gas_limit: String,
    pub pre_verification_gas: String,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub paymaster_and_data: String,
    pub signature: String,
}

#[derive(Serialize)]
pub struct PackedUserOperationResult {
    pub sender: String,
    pub nonce: String,
    pub init_code: String,
    pub call_data: String,
    pub account_gas_limits: String,
    pub pre_verification_gas: String,
    pub gas_fees: String,
    pub paymaster_and_data: String,
    pub signature: String,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum SendUserOperationResult {
    V06(UserOperationResult),
    V07(PackedUserOperationResult),
}

fn pack_account_gas_limits(call_gas_limit: U128, verification_gas_limit: U128) -> FixedBytes<32> {
    let mut packed = [0u8; 32];
    packed[..16].copy_from_slice(&verification_gas_limit.to_be_bytes::<16>());
    packed[16..].copy_from_slice(&call_gas_limit.to_be_bytes::<16>());
    FixedBytes::from(packed)
}

fn pack_gas_fees(max_priority_fee: U128, max_fee_per_gas: U128) -> FixedBytes<32> {
    let mut packed = [0u8; 32];
    packed[..16].copy_from_slice(&max_priority_fee.to_be_bytes::<16>());
    packed[16..].copy_from_slice(&max_fee_per_gas.to_be_bytes::<16>());
    FixedBytes::from(packed)
}

pub fn build_user_operation(params: UserOpParams) -> Result<SendUserOperationResult, String> {
    let sender = params
        .sender
        .parse::<Address>()
        .map_err(|e| create_error(&format!("Invalid sender address: {e}")))?;

    let nonce = params
        .nonce
        .and_then(|n| n.parse::<U256>().ok())
        .unwrap_or(U256::ZERO);

    let init_code = if let Some(init_code) = params.init_code {
        let bytes = alloy::primitives::hex::decode(init_code.trim_start_matches("0x"))
            .map_err(|e| create_error(&format!("Invalid init code: {e}")))?;
        Bytes::from(bytes)
    } else {
        Bytes::new()
    };

    let call_data = if let Some(call_data) = params.call_data {
        let bytes = alloy::primitives::hex::decode(call_data.trim_start_matches("0x"))
            .map_err(|e| create_error(&format!("Invalid call data: {e}")))?;
        Bytes::from(bytes)
    } else {
        Bytes::new()
    };

    let call_gas_limit = params
        .call_gas_limit
        .and_then(|v| v.parse::<U256>().ok())
        .unwrap_or_else(|| U256::from(100000u64));

    let verification_gas_limit = params
        .verification_gas_limit
        .and_then(|v| v.parse::<U256>().ok())
        .unwrap_or_else(|| U256::from(100000u64));

    let pre_verification_gas = params
        .pre_verification_gas
        .and_then(|v| v.parse::<U256>().ok())
        .unwrap_or_else(|| U256::from(21000u64));

    let max_fee_per_gas = params
        .max_fee_per_gas
        .and_then(|v| v.parse::<U256>().ok())
        .unwrap_or_else(|| U256::from(20_000_000_000u64));

    let max_priority_fee_per_gas = params
        .max_priority_fee_per_gas
        .and_then(|v| v.parse::<U256>().ok())
        .unwrap_or_else(|| U256::from(1_000_000_000u64));

    let paymaster_and_data = if let Some(paymaster_and_data) = params.paymaster_and_data {
        let bytes = alloy::primitives::hex::decode(paymaster_and_data.trim_start_matches("0x"))
            .map_err(|e| create_error(&format!("Invalid paymaster and data: {e}")))?;
        Bytes::from(bytes)
    } else {
        Bytes::new()
    };

    let signature = if let Some(signature) = params.signature {
        let bytes = alloy::primitives::hex::decode(signature.trim_start_matches("0x"))
            .map_err(|e| create_error(&format!("Invalid signature: {e}")))?;
        Bytes::from(bytes)
    } else {
        Bytes::new()
    };

    let version = params.entry_point_version.as_deref().unwrap_or("v0.6");

    match version {
        "v0.7" | "0.7" => {
            let call_gas_u128 =
                U128::from_limbs([call_gas_limit.as_limbs()[0], call_gas_limit.as_limbs()[1]]);
            let verification_gas_u128 = U128::from_limbs([
                verification_gas_limit.as_limbs()[0],
                verification_gas_limit.as_limbs()[1],
            ]);
            let max_fee_u128 =
                U128::from_limbs([max_fee_per_gas.as_limbs()[0], max_fee_per_gas.as_limbs()[1]]);
            let max_priority_u128 = U128::from_limbs([
                max_priority_fee_per_gas.as_limbs()[0],
                max_priority_fee_per_gas.as_limbs()[1],
            ]);

            let account_gas_limits = pack_account_gas_limits(call_gas_u128, verification_gas_u128);
            let gas_fees = pack_gas_fees(max_priority_u128, max_fee_u128);

            let packed_op = PackedUserOperation {
                sender,
                nonce,
                init_code: init_code.clone(),
                call_data: call_data.clone(),
                account_gas_limits,
                pre_verification_gas,
                gas_fees,
                paymaster_and_data: paymaster_and_data.clone(),
                signature: signature.clone(),
            };

            Ok(SendUserOperationResult::V07(PackedUserOperationResult {
                sender: format!(
                    "0x{}",
                    alloy::primitives::hex::encode(packed_op.sender.as_slice())
                ),
                nonce: packed_op.nonce.to_string(),
                init_code: format!("0x{}", alloy::primitives::hex::encode(&packed_op.init_code)),
                call_data: format!("0x{}", alloy::primitives::hex::encode(&packed_op.call_data)),
                account_gas_limits: format!(
                    "0x{}",
                    alloy::primitives::hex::encode(packed_op.account_gas_limits.as_slice())
                ),
                pre_verification_gas: packed_op.pre_verification_gas.to_string(),
                gas_fees: format!(
                    "0x{}",
                    alloy::primitives::hex::encode(packed_op.gas_fees.as_slice())
                ),
                paymaster_and_data: format!(
                    "0x{}",
                    alloy::primitives::hex::encode(&packed_op.paymaster_and_data)
                ),
                signature: format!("0x{}", alloy::primitives::hex::encode(&packed_op.signature)),
            }))
        }
        _ => {
            let user_op = UserOperation {
                sender,
                nonce,
                init_code,
                call_data,
                call_gas_limit,
                verification_gas_limit,
                pre_verification_gas,
                max_fee_per_gas,
                max_priority_fee_per_gas,
                paymaster_and_data,
                signature,
            };

            Ok(SendUserOperationResult::V06(UserOperationResult {
                sender: format!(
                    "0x{}",
                    alloy::primitives::hex::encode(user_op.sender.as_slice())
                ),
                nonce: user_op.nonce.to_string(),
                init_code: format!("0x{}", alloy::primitives::hex::encode(&user_op.init_code)),
                call_data: format!("0x{}", alloy::primitives::hex::encode(&user_op.call_data)),
                call_gas_limit: user_op.call_gas_limit.to_string(),
                verification_gas_limit: user_op.verification_gas_limit.to_string(),
                pre_verification_gas: user_op.pre_verification_gas.to_string(),
                max_fee_per_gas: user_op.max_fee_per_gas.to_string(),
                max_priority_fee_per_gas: user_op.max_priority_fee_per_gas.to_string(),
                paymaster_and_data: format!(
                    "0x{}",
                    alloy::primitives::hex::encode(&user_op.paymaster_and_data)
                ),
                signature: format!("0x{}", alloy::primitives::hex::encode(&user_op.signature)),
            }))
        }
    }
}

pub fn build_send_user_operation(params: UserOpParams) -> Result<SendUserOperation, String> {
    let sender = params
        .sender
        .parse::<Address>()
        .map_err(|e| create_error(&format!("Invalid sender address: {e}")))?;

    let nonce = params
        .nonce
        .and_then(|n| n.parse::<U256>().ok())
        .unwrap_or(U256::ZERO);

    let init_code = if let Some(init_code) = params.init_code {
        let bytes = alloy::primitives::hex::decode(init_code.trim_start_matches("0x"))
            .map_err(|e| create_error(&format!("Invalid init code: {e}")))?;
        Bytes::from(bytes)
    } else {
        Bytes::new()
    };

    let call_data = if let Some(call_data) = params.call_data {
        let bytes = alloy::primitives::hex::decode(call_data.trim_start_matches("0x"))
            .map_err(|e| create_error(&format!("Invalid call data: {e}")))?;
        Bytes::from(bytes)
    } else {
        Bytes::new()
    };

    let call_gas_limit = params
        .call_gas_limit
        .and_then(|v| v.parse::<U256>().ok())
        .unwrap_or_else(|| U256::from(100000u64));

    let verification_gas_limit = params
        .verification_gas_limit
        .and_then(|v| v.parse::<U256>().ok())
        .unwrap_or_else(|| U256::from(100000u64));

    let pre_verification_gas = params
        .pre_verification_gas
        .and_then(|v| v.parse::<U256>().ok())
        .unwrap_or_else(|| U256::from(21000u64));

    let max_fee_per_gas = params
        .max_fee_per_gas
        .and_then(|v| v.parse::<U256>().ok())
        .unwrap_or_else(|| U256::from(20_000_000_000u64));

    let max_priority_fee_per_gas = params
        .max_priority_fee_per_gas
        .and_then(|v| v.parse::<U256>().ok())
        .unwrap_or_else(|| U256::from(1_000_000_000u64));

    let paymaster_and_data = if let Some(paymaster_and_data) = params.paymaster_and_data {
        let bytes = alloy::primitives::hex::decode(paymaster_and_data.trim_start_matches("0x"))
            .map_err(|e| create_error(&format!("Invalid paymaster and data: {e}")))?;
        Bytes::from(bytes)
    } else {
        Bytes::new()
    };

    let signature = if let Some(signature) = params.signature {
        let bytes = alloy::primitives::hex::decode(signature.trim_start_matches("0x"))
            .map_err(|e| create_error(&format!("Invalid signature: {e}")))?;
        Bytes::from(bytes)
    } else {
        Bytes::new()
    };

    let version = params.entry_point_version.as_deref().unwrap_or("v0.6");

    match version {
        "v0.7" | "0.7" => {
            let call_gas_u128 =
                U128::from_limbs([call_gas_limit.as_limbs()[0], call_gas_limit.as_limbs()[1]]);
            let verification_gas_u128 = U128::from_limbs([
                verification_gas_limit.as_limbs()[0],
                verification_gas_limit.as_limbs()[1],
            ]);
            let max_fee_u128 =
                U128::from_limbs([max_fee_per_gas.as_limbs()[0], max_fee_per_gas.as_limbs()[1]]);
            let max_priority_u128 = U128::from_limbs([
                max_priority_fee_per_gas.as_limbs()[0],
                max_priority_fee_per_gas.as_limbs()[1],
            ]);

            let account_gas_limits = pack_account_gas_limits(call_gas_u128, verification_gas_u128);
            let gas_fees = pack_gas_fees(max_priority_u128, max_fee_u128);

            Ok(SendUserOperation::EntryPointV07 {
                packed_user_op: PackedUserOperation {
                    sender,
                    nonce,
                    init_code,
                    call_data,
                    account_gas_limits,
                    pre_verification_gas,
                    gas_fees,
                    paymaster_and_data,
                    signature,
                },
            })
        }
        _ => Ok(SendUserOperation::EntryPointV06 {
            user_op: UserOperation {
                sender,
                nonce,
                init_code,
                call_data,
                call_gas_limit,
                verification_gas_limit,
                pre_verification_gas,
                max_fee_per_gas,
                max_priority_fee_per_gas,
                paymaster_and_data,
                signature,
            },
        }),
    }
}
