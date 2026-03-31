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
use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::guardian::recovery::initialize::{
    initialize_recovery, InitializeRecoveryParams,
};
use crate::account::modular_smart_account::guardian::recovery::RecoveryType;

/// Initialize a recovery process for a smart account
///
/// This function allows a guardian to initialize a recovery process.
/// The recovery must be finalized after REQUEST_DELAY_TIME (24 hours) has passed.
///
/// # Parameters
/// * `rpc_url` - RPC URL for the blockchain network
/// * `guardian_executor` - Address of the GuardianExecutor contract
/// * `account` - Address of the smart account
/// * `recovery_type` - Type of recovery (None, EOA, or Passkey)
/// * `data` - Recovery data (hex-encoded bytes, 0x-prefixed)
/// * `guardian_private_key` - Private key of the guardian (0x-prefixed hex string)
///
/// # Returns
/// Promise that resolves to the transaction receipt hash as a hex string
#[wasm_bindgen]
pub fn initialize_recovery_wasm(
    rpc_url: String,
    guardian_executor: String,
    account: String,
    recovery_type: RecoveryType,
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

        // Convert recovery type
        let core_recovery_type: zksync_sso_erc4337_core::erc4337::account::modular_smart_account::guardian::recovery::RecoveryType = recovery_type.into();

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
        match initialize_recovery(InitializeRecoveryParams {
            guardian_executor: guardian_executor_addr,
            account: account_addr,
            recovery_type: core_recovery_type,
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
                "Failed to initialize recovery: {}",
                e
            ))),
        }
    })
}
