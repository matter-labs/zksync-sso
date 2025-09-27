use alloy_web_core::erc4337::{
    account::{DeploymentConfig, deploy_simple_account as deploy_account_core},
    infrastructure::{InfrastructureConfig, deploy_infrastructure as deploy_infra_core},
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct InfrastructureDeploymentResult {
    pub entry_point: String,
    pub factory: String,
}

#[derive(Serialize, Deserialize)]
pub struct InfrastructureAccountDeploymentResult {
    pub factory_address: String,
    pub account_address: String,
    pub owner: String,
    pub entry_point: String,
}

#[derive(Deserialize)]
pub struct DeployInfrastructureParams {
    pub rpc_url: Option<String>,
    pub private_key: Option<String>,
}

#[derive(Deserialize)]
pub struct DeployAccountParams {
    pub rpc_url: Option<String>,
    pub private_key: Option<String>,
    pub factory_address: String,
    pub entry_point_address: Option<String>,
    pub owner_address: Option<String>,
    pub salt: Option<u64>,
}

/// Deploy ERC-4337 infrastructure (EntryPoint and SimpleAccountFactory)
#[wasm_bindgen]
pub async fn deploy_infrastructure(params: JsValue) -> Result<JsValue, JsValue> {
    let params: DeployInfrastructureParams = serde_wasm_bindgen::from_value(params)
        .map_err(|e| JsValue::from_str(&format!("Invalid parameters: {e}")))?;

    let config = InfrastructureConfig {
        rpc_url: params
            .rpc_url
            .unwrap_or_else(|| "http://localhost:8545".to_string()),
        private_key: params.private_key.unwrap_or_else(|| {
            // Default Anvil private key
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
        }),
    };

    match deploy_infra_core(config).await {
        Ok(deployment) => {
            let result = InfrastructureDeploymentResult {
                entry_point: format!("0x{:x}", deployment.entry_point),
                factory: format!("0x{:x}", deployment.factory),
            };
            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
        }
        Err(e) => Err(JsValue::from_str(&format!("Deployment failed: {e}"))),
    }
}

/// Deploy a SimpleAccount instance
#[wasm_bindgen]
pub async fn deploy_simple_account(params: JsValue) -> Result<JsValue, JsValue> {
    let params: DeployAccountParams = serde_wasm_bindgen::from_value(params)
        .map_err(|e| JsValue::from_str(&format!("Invalid parameters: {e}")))?;

    let config = DeploymentConfig {
        rpc_url: params
            .rpc_url
            .unwrap_or_else(|| "http://localhost:8545".to_string()),
        private_key: params.private_key.unwrap_or_else(|| {
            // Default Anvil private key
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
        }),
        factory_address: Some(params.factory_address),
        entry_point_address: params.entry_point_address,
        owner_address: params.owner_address,
        salt: params.salt,
    };

    match deploy_account_core(config).await {
        Ok(deployment) => {
            let result = InfrastructureAccountDeploymentResult {
                factory_address: format!("0x{:x}", deployment.factory_address),
                account_address: format!("0x{:x}", deployment.account_address),
                owner: format!("0x{:x}", deployment.owner),
                entry_point: format!("0x{:x}", deployment.entry_point),
            };
            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
        }
        Err(e) => Err(JsValue::from_str(&format!(
            "Account deployment failed: {e}"
        ))),
    }
}
