use crate::wasm_transport::WasmHttpTransport;
use alloy::{
    network::EthereumWallet,
    primitives::{Address, Bytes},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};
use alloy_rpc_client::RpcClient;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::guardian::recovery::finalize::{
    finalize_recovery, FinalizeRecoveryParams,
};

/// Finalize a recovery process for a smart account
///
/// This function allows a guardian to finalize a recovery that was previously initialized.
/// The recovery must have been initialized at least REQUEST_DELAY_TIME (24 hours) ago.
///
/// # Parameters
/// * `rpc_url` - RPC URL for the blockchain network
/// * `guardian_executor` - Address of the GuardianExecutor contract
/// * `account` - Address of the smart account
/// * `data` - Recovery data (hex-encoded bytes, 0x-prefixed)
/// * `guardian_private_key` - Private key of the guardian (0x-prefixed hex string)
///
/// # Returns
/// Promise that resolves to the transaction receipt hash as a hex string
#[wasm_bindgen]
pub fn finalize_recovery_wasm(
    rpc_url: String,
    guardian_executor: String,
    account: String,
    data: String,
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

        // Parse data
        let data_bytes =
            hex::decode(data.trim_start_matches("0x")).map_err(|e| {
                JsValue::from_str(&format!("Invalid data hex: {}", e))
            })?;
        let data_bytes: Bytes = data_bytes.into();

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
        match finalize_recovery(FinalizeRecoveryParams {
            guardian_executor: guardian_executor_addr,
            account: account_addr,
            data: data_bytes,
            guardian_provider,
        })
        .await
        {
            Ok(receipt) => {
                let tx_hash = receipt.transaction_hash;
                Ok(JsValue::from_str(&format!("0x{:x}", tx_hash)))
            }
            Err(e) => Err(JsValue::from_str(&format!(
                "Failed to finalize recovery: {}",
                e
            ))),
        }
    })
}
