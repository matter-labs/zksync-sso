use crate::wasm_transport::WasmHttpTransport;
use alloy::{
    network::EthereumWallet,
    primitives::Address,
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};
use alloy_rpc_client::RpcClient;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::guardian::accept::{
    accept_guardian, AcceptGuardianParams,
};

/// Accept a proposed guardian for a smart account
///
/// This function allows a guardian to accept their proposed role for a smart account.
/// The guardian must sign the transaction using their private key.
///
/// # Parameters
/// * `rpc_url` - RPC URL for the blockchain network
/// * `guardian_executor` - Address of the GuardianExecutor contract
/// * `account` - Address of the smart account
/// * `guardian_private_key` - Private key of the guardian (0x-prefixed hex string)
///
/// # Returns
/// Promise that resolves to the transaction receipt hash as a hex string
#[wasm_bindgen]
pub fn accept_guardian_wasm(
    rpc_url: String,
    guardian_executor: String,
    account: String,
    guardian_private_key: String,
) -> js_sys::Promise {
    future_to_promise(async move {
        // Parse addresses
        let guardian_executor_addr = match guardian_executor.parse::<Address>()
        {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid guardian executor address: {}",
                    e
                )));
            }
        };

        let account_addr = match account.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid account address: {}",
                    e
                )));
            }
        };

        // Parse guardian private key
        let guardian_key = match guardian_private_key
            .trim_start_matches("0x")
            .parse::<PrivateKeySigner>()
        {
            Ok(signer) => signer,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid guardian private key: {}",
                    e
                )));
            }
        };

        let guardian_wallet = EthereumWallet::from(guardian_key);

        // Create transport and provider with wallet
        let transport = WasmHttpTransport::new(rpc_url);
        let client = RpcClient::new(transport.clone(), false);
        let guardian_provider = ProviderBuilder::new()
            .wallet(guardian_wallet)
            .connect_client(client);

        // Call the core function
        match accept_guardian(AcceptGuardianParams {
            guardian_executor: guardian_executor_addr,
            account: account_addr,
            guardian_provider,
        })
        .await
        {
            Ok(receipt) => {
                let tx_hash = receipt.transaction_hash;
                Ok(JsValue::from_str(&format!("0x{:x}", tx_hash)))
            }
            Err(e) => Err(JsValue::from_str(&format!(
                "Failed to accept guardian: {}",
                e
            ))),
        }
    })
}
