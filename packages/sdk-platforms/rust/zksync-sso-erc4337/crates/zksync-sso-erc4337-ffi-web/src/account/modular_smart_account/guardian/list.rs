use crate::wasm_transport::WasmHttpTransport;
use alloy::primitives::Address;
use alloy::providers::ProviderBuilder;
use alloy_rpc_client::RpcClient;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::guardian::list::get_guardians_list;

/// Get the list of guardians for a smart account
///
/// # Parameters
/// * `rpc_url` - RPC URL for the blockchain network
/// * `account_address` - Address of the smart account
/// * `guardian_executor_address` - Address of the GuardianExecutor contract
///
/// # Returns
/// Promise that resolves to a JSON array of guardian addresses (hex strings)
#[wasm_bindgen]
pub fn get_guardians_list_wasm(
    rpc_url: String,
    account_address: String,
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
        match get_guardians_list(account_addr, guardian_executor_addr, provider)
            .await
        {
            Ok(guardians) => {
                let addresses: Vec<String> = guardians
                    .iter()
                    .map(|addr| format!("0x{:x}", addr))
                    .collect();
                let json = serde_json::to_string(&addresses).map_err(|e| {
                    JsValue::from_str(&format!(
                        "Failed to serialize guardians list: {}",
                        e
                    ))
                })?;
                Ok(JsValue::from_str(&json))
            }
            Err(e) => Err(JsValue::from_str(&format!(
                "Failed to get guardians list: {}",
                e
            ))),
        }
    })
}
