use crate::wasm_transport::WasmHttpTransport;
use alloy::{primitives::Address, providers::ProviderBuilder};
use alloy_rpc_client::RpcClient;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use zksync_sso_erc4337_core::{
    config::contracts::Contracts as CoreContracts,
    erc4337::account::modular_smart_account::session::active::get_active_sessions,
};

/// Get the list of active sessions for a smart account
///
/// # Parameters
/// * `rpc_url` - RPC URL for the blockchain network
/// * `account_address` - Address of the smart account
/// * `contracts_json` - JSON string containing contract addresses (entryPoint, accountFactory, webauthnValidator, eoaValidator, sessionValidator, guardianExecutor)
///
/// # Returns
/// Promise that resolves to a JSON array of active sessions with session_hash and session_spec
#[wasm_bindgen]
pub fn get_active_sessions_wasm(
    rpc_url: String,
    account_address: String,
    contracts_json: String,
) -> js_sys::Promise {
    future_to_promise(async move {
        // Parse account address
        let account_addr = match account_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid account address: {}",
                    e
                )));
            }
        };

        // Parse contracts JSON
        let contracts: CoreContracts =
            match serde_json::from_str(&contracts_json) {
                Ok(c) => c,
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Invalid contracts JSON: {}",
                        e
                    )));
                }
            };

        // Create transport and provider
        let transport = WasmHttpTransport::new(rpc_url);
        let client = RpcClient::new(transport.clone(), false);
        let provider = ProviderBuilder::new().connect_client(client);

        // Call the core function
        match get_active_sessions(account_addr, provider, contracts).await {
            Ok(sessions) => {
                let json = serde_json::to_string(&sessions).map_err(|e| {
                    JsValue::from_str(&format!(
                        "Failed to serialize active sessions: {}",
                        e
                    ))
                })?;
                Ok(JsValue::from_str(&json))
            }
            Err(e) => Err(JsValue::from_str(&format!(
                "Failed to get active sessions: {}",
                e
            ))),
        }
    })
}
