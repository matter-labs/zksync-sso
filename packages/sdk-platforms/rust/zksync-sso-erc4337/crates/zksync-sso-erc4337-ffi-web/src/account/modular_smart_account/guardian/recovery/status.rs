use crate::wasm_transport::WasmHttpTransport;
use alloy::primitives::Address;
use alloy::providers::ProviderBuilder;
use alloy_rpc_client::RpcClient;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::guardian::recovery::status::{
    get_recovery_status, RecoveryStatus as CoreRecoveryStatus,
};

/// Recovery status enum for WASM
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryStatus {
    Initialized,
    Finalized,
}

impl From<CoreRecoveryStatus> for RecoveryStatus {
    fn from(status: CoreRecoveryStatus) -> Self {
        match status {
            CoreRecoveryStatus::Initialized => RecoveryStatus::Initialized,
            CoreRecoveryStatus::Finalized => RecoveryStatus::Finalized,
        }
    }
}

/// Get the recovery status for a smart account and guardian
///
/// # Parameters
/// * `rpc_url` - RPC URL for the blockchain network
/// * `account_address` - Address of the smart account
/// * `guardian_address` - Address of the guardian
/// * `guardian_executor_address` - Address of the GuardianExecutor contract
///
/// # Returns
/// Promise that resolves to a JSON string with the recovery status, or null if no recovery is in progress
#[wasm_bindgen]
pub fn get_recovery_status_wasm(
    rpc_url: String,
    account_address: String,
    guardian_address: String,
    guardian_executor_address: String,
) -> js_sys::Promise {
    future_to_promise(async move {
        // Parse addresses
        let account_addr = match account_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid account address: {}",
                    e
                )));
            }
        };

        let guardian_addr = match guardian_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid guardian address: {}",
                    e
                )));
            }
        };

        let guardian_executor_addr =
            match guardian_executor_address.parse::<Address>() {
                Ok(addr) => addr,
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Invalid guardian executor address: {}",
                        e
                    )));
                }
            };

        // Create transport and provider
        let transport = WasmHttpTransport::new(rpc_url);
        let client = RpcClient::new(transport.clone(), false);
        let provider = ProviderBuilder::new().connect_client(client);

        // Call the core function
        match get_recovery_status(
            account_addr,
            guardian_addr,
            guardian_executor_addr,
            provider,
        )
        .await
        {
            Ok(Some(status)) => {
                let wasm_status: RecoveryStatus = status.into();
                let json =
                    serde_json::to_string(&wasm_status).map_err(|e| {
                        JsValue::from_str(&format!(
                            "Failed to serialize recovery status: {}",
                            e
                        ))
                    })?;
                Ok(JsValue::from_str(&json))
            }
            Ok(None) => Ok(JsValue::NULL),
            Err(e) => Err(JsValue::from_str(&format!(
                "Failed to get recovery status: {}",
                e
            ))),
        }
    })
}
