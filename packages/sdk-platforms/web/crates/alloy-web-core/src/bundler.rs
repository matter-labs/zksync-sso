use alloy::primitives::Address;
use alloy::rpc::types::eth::UserOperation;
use eyre::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct BundlerClient {
    pub url: String,
    pub entry_point: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserOperationReceipt {
    #[serde(rename = "userOpHash")]
    pub user_op_hash: String,
    #[serde(rename = "entryPoint")]
    pub entry_point: Address,
    pub sender: Address,
    pub nonce: String,
    pub paymaster: Option<Address>,
    #[serde(rename = "actualGasCost")]
    pub actual_gas_cost: String,
    #[serde(rename = "actualGasUsed")]
    pub actual_gas_used: String,
    pub success: bool,
    pub reason: Option<String>,
    pub logs: Vec<serde_json::Value>,
    pub receipt: serde_json::Value,
}

impl BundlerClient {
    pub fn new(url: String, entry_point: Address) -> Self {
        Self { url, entry_point }
    }

    pub async fn send_user_operation(&self, user_op: &UserOperation) -> Result<String> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_sendUserOperation",
            "params": [user_op, self.entry_point],
            "id": 1
        });

        let client = Client::new();
        let response = client.post(&self.url).json(&request_body).send().await?;

        let response_text = response.text().await?;
        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;

        if let Some(error) = response_json.get("error") {
            return Err(eyre::eyre!("Bundler rejected UserOperation: {}", error));
        }

        if let Some(result) = response_json.get("result") {
            if let Some(hash) = result.as_str() {
                return Ok(hash.to_string());
            }
        }

        Err(eyre::eyre!("Invalid response from bundler"))
    }

    pub async fn get_user_operation_by_hash(&self, hash: &str) -> Result<Option<UserOperation>> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getUserOperationByHash",
            "params": [hash],
            "id": 1
        });

        let client = Client::new();
        let response = client.post(&self.url).json(&request_body).send().await?;

        let response_text = response.text().await?;
        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;

        if let Some(error) = response_json.get("error") {
            return Err(eyre::eyre!("Bundler error: {}", error));
        }

        if let Some(result) = response_json.get("result") {
            if result.is_null() {
                return Ok(None);
            }
            let user_op: UserOperation = serde_json::from_value(result.clone())?;
            return Ok(Some(user_op));
        }

        Ok(None)
    }

    pub async fn get_user_operation_receipt(
        &self,
        hash: &str,
    ) -> Result<Option<UserOperationReceipt>> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getUserOperationReceipt",
            "params": [hash],
            "id": 1
        });

        let client = Client::new();
        let response = client.post(&self.url).json(&request_body).send().await?;

        let response_text = response.text().await?;
        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;

        if let Some(error) = response_json.get("error") {
            return Err(eyre::eyre!("Bundler error: {}", error));
        }

        if let Some(result) = response_json.get("result") {
            if result.is_null() {
                return Ok(None);
            }
            let receipt: UserOperationReceipt = serde_json::from_value(result.clone())?;
            return Ok(Some(receipt));
        }

        Ok(None)
    }

    pub async fn estimate_user_operation_gas(
        &self,
        user_op: &UserOperation,
    ) -> Result<serde_json::Value> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_estimateUserOperationGas",
            "params": [user_op, self.entry_point],
            "id": 1
        });

        let client = Client::new();
        let response = client.post(&self.url).json(&request_body).send().await?;

        let response_text = response.text().await?;
        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;

        if let Some(error) = response_json.get("error") {
            return Err(eyre::eyre!("Gas estimation failed: {}", error));
        }

        if let Some(result) = response_json.get("result") {
            return Ok(result.clone());
        }

        Err(eyre::eyre!("Invalid gas estimation response"))
    }
}
