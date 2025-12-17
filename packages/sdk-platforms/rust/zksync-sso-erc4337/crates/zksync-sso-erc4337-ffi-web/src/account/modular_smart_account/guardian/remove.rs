use crate::wasm_transport::WasmHttpTransport;
use alloy::{
    network::EthereumWallet, primitives::Address, providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};
use alloy_rpc_client::RpcClient;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use zksync_sso_erc4337_core::erc4337::{
    account::modular_smart_account::guardian::remove::{
        RemoveGuardianParams, remove_guardian,
    },
    bundler::{config::BundlerConfig, pimlico::client::BundlerClient},
    signer::create_eoa_signer,
};

/// Remove a guardian from a smart account
///
/// This function creates a user operation to remove an active guardian.
/// The account owner must sign the transaction.
///
/// # Parameters
/// * `config` - SendTransactionConfig with RPC URL, bundler URL, and entry point
/// * `guardian_executor` - Address of the GuardianExecutor contract
/// * `guardian_to_remove` - Address of the guardian to remove
/// * `account_address` - Address of the smart account
/// * `eoa_validator_address` - Address of the EOA validator module
/// * `eoa_private_key` - Private key of the account owner (0x-prefixed hex string)
///
/// # Returns
/// Promise that resolves to the user operation receipt hash as a hex string
#[wasm_bindgen]
pub fn remove_guardian_wasm(
    config: crate::SendTransactionConfig,
    guardian_executor: String,
    guardian_to_remove: String,
    account_address: String,
    eoa_validator_address: String,
    eoa_private_key: String,
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

        let guardian_to_remove_addr =
            match guardian_to_remove.parse::<Address>() {
                Ok(addr) => addr,
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Invalid guardian to remove address: {}",
                        e
                    )));
                }
            };

        let account_addr = match account_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid account address: {}",
                    e
                )));
            }
        };

        let entry_point_addr =
            match config.entry_point_address().parse::<Address>() {
                Ok(addr) => addr,
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Invalid entry point address: {}",
                        e
                    )));
                }
            };

        let eoa_validator_addr = match eoa_validator_address.parse::<Address>()
        {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid EOA validator address: {}",
                    e
                )));
            }
        };

        // Parse EOA private key
        let eoa_key = match eoa_private_key
            .trim_start_matches("0x")
            .parse::<PrivateKeySigner>()
        {
            Ok(signer) => signer,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid EOA private key: {}",
                    e
                )));
            }
        };

        let eoa_wallet = EthereumWallet::from(eoa_key.clone());

        // Create transport and provider
        let transport = WasmHttpTransport::new(config.rpc_url());
        let client = RpcClient::new(transport.clone(), false);
        let provider =
            ProviderBuilder::new().wallet(eoa_wallet).connect_client(client);

        // Create bundler client
        let bundler_client = {
            let bundler_config = BundlerConfig::new(config.bundler_url());
            BundlerClient::new(bundler_config)
        };

        // Create EOA signer
        let eoa_key_str = format!("0x{}", hex::encode(eoa_key.to_bytes()));
        let signer = match create_eoa_signer(eoa_key_str, eoa_validator_addr) {
            Ok(signer) => signer,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Failed to create EOA signer: {}",
                    e
                )));
            }
        };

        // Call the core function
        match remove_guardian(RemoveGuardianParams {
            guardian_executor: guardian_executor_addr,
            guardian_to_remove: guardian_to_remove_addr,
            account_address: account_addr,
            entry_point_address: entry_point_addr,
            paymaster: None,
            provider,
            bundler_client,
            signer,
        })
        .await
        {
            Ok(receipt) => {
                let tx_hash = receipt.receipt.transaction_hash;
                Ok(JsValue::from_str(&tx_hash))
            }
            Err(e) => Err(JsValue::from_str(&format!(
                "Failed to remove guardian: {}",
                e
            ))),
        }
    })
}
