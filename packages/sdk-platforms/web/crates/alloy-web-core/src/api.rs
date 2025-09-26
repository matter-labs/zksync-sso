use crate::bundler::BundlerClient;
use crate::user_op_signer::UserOperationSigner;
use alloy::primitives::{Address, Bytes, U256};
use alloy::rpc::types::eth::UserOperation;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

// High-level API types that don't expose Alloy internals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOpRequest {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOpResponse {
    pub hash: String,
    pub user_operation: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOpReceipt {
    pub user_op_hash: String,
    pub entry_point: String,
    pub sender: String,
    pub nonce: String,
    pub paymaster: Option<String>,
    pub actual_gas_cost: String,
    pub actual_gas_used: String,
    pub success: bool,
    pub reason: Option<String>,
    pub transaction_hash: Option<String>,
    pub block_number: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BundlerConfig {
    pub url: String,
    pub entry_point: String,
    pub chain_id: u64,
}

/// High-level API for UserOperation management
pub struct UserOperationAPI;

impl UserOperationAPI {
    /// Sign a UserOperation with a private key
    pub async fn sign_user_operation(
        user_op: UserOpRequest,
        private_key: String,
        config: BundlerConfig,
    ) -> Result<serde_json::Value> {
        // Convert to Alloy types internally
        let entry_point = Address::from_str(&config.entry_point)?;
        let alloy_user_op = Self::convert_to_alloy_user_op(user_op)?;

        // Sign using our internal signer
        let signer = UserOperationSigner::new(&private_key, entry_point, config.chain_id)?;
        let signed_op = signer.sign_user_operation(alloy_user_op).await?;

        // Convert back to JSON for CLI consumption
        Ok(serde_json::to_value(signed_op)?)
    }

    /// Submit a UserOperation to a bundler
    pub async fn submit_user_operation(
        user_op: serde_json::Value,
        config: BundlerConfig,
    ) -> Result<String> {
        let entry_point = Address::from_str(&config.entry_point)?;
        let alloy_user_op: UserOperation = serde_json::from_value(user_op)?;

        let client = BundlerClient::new(config.url, entry_point);
        client.send_user_operation(&alloy_user_op).await
    }

    /// Sign and submit a UserOperation in one call
    pub async fn sign_and_submit_user_operation(
        user_op: UserOpRequest,
        private_key: String,
        config: BundlerConfig,
    ) -> Result<String> {
        // Sign first
        let signed_op = Self::sign_user_operation(user_op, private_key, config.clone()).await?;

        // Then submit
        Self::submit_user_operation(signed_op, config).await
    }

    /// Get UserOperation receipt by hash
    pub async fn get_user_operation_receipt(
        hash: String,
        bundler_url: String,
    ) -> Result<Option<UserOpReceipt>> {
        let client = BundlerClient::new(bundler_url, Address::ZERO);

        match client.get_user_operation_receipt(&hash).await? {
            Some(receipt) => {
                Ok(Some(UserOpReceipt {
                    user_op_hash: receipt.user_op_hash,
                    entry_point: receipt.entry_point.to_string(),
                    sender: receipt.sender.to_string(),
                    nonce: receipt.nonce,
                    paymaster: receipt.paymaster.map(|p| p.to_string()),
                    actual_gas_cost: receipt.actual_gas_cost,
                    actual_gas_used: receipt.actual_gas_used,
                    success: receipt.success,
                    reason: receipt.reason,
                    transaction_hash: None, // Would need to be extracted from receipt
                    block_number: None,     // Would need to be extracted from receipt
                }))
            }
            None => Ok(None),
        }
    }

    /// Estimate gas for a UserOperation
    pub async fn estimate_user_operation_gas(
        user_op: UserOpRequest,
        config: BundlerConfig,
    ) -> Result<HashMap<String, String>> {
        let entry_point = Address::from_str(&config.entry_point)?;
        let alloy_user_op = Self::convert_to_alloy_user_op(user_op)?;

        let client = BundlerClient::new(config.url, entry_point);
        let gas_estimate = client.estimate_user_operation_gas(&alloy_user_op).await?;

        // Convert to simple string map
        let mut result = HashMap::new();
        if let serde_json::Value::Object(map) = gas_estimate {
            for (key, value) in map {
                result.insert(key, value.to_string());
            }
        }

        Ok(result)
    }

    /// Internal helper to convert API types to Alloy types
    fn convert_to_alloy_user_op(
        user_op: UserOpRequest,
    ) -> Result<alloy::rpc::types::eth::UserOperation> {
        let sender = Address::from_str(&user_op.sender)?;
        let nonce = user_op
            .nonce
            .and_then(|n| n.parse::<U256>().ok())
            .unwrap_or(U256::ZERO);

        let init_code = if let Some(init_code) = user_op.init_code {
            let bytes = alloy::primitives::hex::decode(init_code.trim_start_matches("0x"))?;
            Bytes::from(bytes)
        } else {
            Bytes::new()
        };

        let call_data = if let Some(call_data) = user_op.call_data {
            let bytes = alloy::primitives::hex::decode(call_data.trim_start_matches("0x"))?;
            Bytes::from(bytes)
        } else {
            Bytes::new()
        };

        let call_gas_limit = user_op
            .call_gas_limit
            .and_then(|v| v.parse::<U256>().ok())
            .unwrap_or_else(|| U256::from(100000u64));

        let verification_gas_limit = user_op
            .verification_gas_limit
            .and_then(|v| v.parse::<U256>().ok())
            .unwrap_or_else(|| U256::from(100000u64));

        let pre_verification_gas = user_op
            .pre_verification_gas
            .and_then(|v| v.parse::<U256>().ok())
            .unwrap_or_else(|| U256::from(21000u64));

        let max_fee_per_gas = user_op
            .max_fee_per_gas
            .and_then(|v| v.parse::<U256>().ok())
            .unwrap_or_else(|| U256::from(20_000_000_000u64));

        let max_priority_fee_per_gas = user_op
            .max_priority_fee_per_gas
            .and_then(|v| v.parse::<U256>().ok())
            .unwrap_or_else(|| U256::from(1_000_000_000u64));

        let paymaster_and_data = if let Some(paymaster_and_data) = user_op.paymaster_and_data {
            let bytes =
                alloy::primitives::hex::decode(paymaster_and_data.trim_start_matches("0x"))?;
            Bytes::from(bytes)
        } else {
            Bytes::new()
        };

        let signature = if let Some(signature) = user_op.signature {
            let bytes = alloy::primitives::hex::decode(signature.trim_start_matches("0x"))?;
            Bytes::from(bytes)
        } else {
            Bytes::new()
        };

        Ok(alloy::rpc::types::eth::UserOperation {
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
        })
    }
}
