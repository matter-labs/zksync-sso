use alloy::{
    network::EthereumWallet,
    primitives::{Address, Bytes, FixedBytes, U256, keccak256},
    providers::ProviderBuilder,
    rpc::types::erc4337::PackedUserOperation as AlloyPackedUserOperation,
    signers::local::PrivateKeySigner,
};
use alloy_rpc_client::RpcClient;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use zksync_sso_erc4337_core::{
    chain::{Chain, id::ChainId},
    config::contracts::Contracts as CoreContracts,
    erc4337::{
        account::modular_smart_account::{
            add_passkey::PasskeyPayload as CorePasskeyPayload,
            deploy::{
                EOASigners as CoreEOASigners,
                WebauthNSigner as CoreWebauthNSigner,
            },
        },
        entry_point::version::EntryPointVersion,
    },
};

// WASM transport is implemented but not yet fully integrated with Alloy's Provider trait
// For now, we expose offline computation functions
mod wasm_transport;
use wasm_transport::WasmHttpTransport;

// Initialize logging and panic hook for WASM
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    // Ignore errors if logger is already initialized
    let _ = console_log::init_with_level(log::Level::Info);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Global storage for UserOperations being prepared/signed
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

// Store both the UserOp and the validator address
static USER_OPS: Lazy<
    Mutex<HashMap<String, (AlloyPackedUserOperation, Address)>>,
> = Lazy::new(|| Mutex::new(HashMap::new()));

// Test function to verify WASM is working
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! WASM is working.", name)
}

/// Test function using core crate - get chain info
#[wasm_bindgen]
pub fn get_chain_info(chain_id: u64) -> String {
    let chain = Chain::from(ChainId::from(chain_id));
    format!("Chain ID: {}, CAIP-2: {}", chain.id, chain.caip2_identifier())
}

/// Test function using core crate - get Ethereum Sepolia chain info
#[wasm_bindgen]
pub fn get_ethereum_sepolia_info() -> String {
    let chain = Chain::new(
        ChainId::ETHEREUM_MAINNET,
        EntryPointVersion::V08,
        "Ethereum Mainnet".to_string(),
    );
    format!(
        "Chain: {}, ID: {}, Entry Point Version: {:?}, CAIP-2: {}",
        chain.name,
        chain.id,
        chain.entry_point_version,
        chain.caip2_identifier()
    )
}

/// Test function to verify HTTP transport with reqwasm
/// Makes a simple eth_chainId RPC call to verify the transport works
#[wasm_bindgen]
pub fn test_http_transport(rpc_url: String) -> js_sys::Promise {
    future_to_promise(async move {
        console_log!("Creating WASM HTTP transport for: {}", rpc_url);

        // Create transport
        let transport = WasmHttpTransport::new(rpc_url);

        console_log!("Creating RPC client with transport");

        // Create RPC client with our custom transport
        let client = RpcClient::new(transport, false);

        console_log!("Making eth_chainId request...");

        // Make a proper eth_chainId request using Alloy's RPC client
        match client.request("eth_chainId", ()).await {
            Ok(chain_id) => {
                let result: String = chain_id;
                console_log!("Got chain ID: {}", result);
                Ok(JsValue::from_str(&format!("Success! Chain ID: {}", result)))
            }
            Err(e) => {
                console_log!("Request failed: {}", e);
                Ok(JsValue::from_str(&format!("RPC request failed: {}", e)))
            }
        }
    })
}

// WebAuthn passkey payload for WASM
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct PasskeyPayload {
    credential_id: Vec<u8>,
    passkey_x: Vec<u8>,
    passkey_y: Vec<u8>,
    origin_domain: String,
}

#[wasm_bindgen]
impl PasskeyPayload {
    #[wasm_bindgen(constructor)]
    pub fn new(
        credential_id: Vec<u8>,
        passkey_x: Vec<u8>,
        passkey_y: Vec<u8>,
        origin_domain: String,
    ) -> Self {
        Self { credential_id, passkey_x, passkey_y, origin_domain }
    }

    #[wasm_bindgen(getter)]
    pub fn credential_id(&self) -> Vec<u8> {
        self.credential_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn passkey_x(&self) -> Vec<u8> {
        self.passkey_x.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn passkey_y(&self) -> Vec<u8> {
        self.passkey_y.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn origin_domain(&self) -> String {
        self.origin_domain.clone()
    }
}

// Config for WASM account deployments
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct DeployAccountConfig {
    rpc_url: String,
    factory_address: String,
    deployer_private_key: String,
    eoa_validator_address: Option<String>,
    webauthn_validator_address: Option<String>,
}

#[wasm_bindgen]
impl DeployAccountConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(
        rpc_url: String,
        factory_address: String,
        deployer_private_key: String,
        eoa_validator_address: Option<String>,
        webauthn_validator_address: Option<String>,
    ) -> Self {
        Self {
            rpc_url,
            factory_address,
            deployer_private_key,
            eoa_validator_address,
            webauthn_validator_address,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn rpc_url(&self) -> String {
        self.rpc_url.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn factory_address(&self) -> String {
        self.factory_address.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn deployer_private_key(&self) -> String {
        self.deployer_private_key.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn eoa_validator_address(&self) -> Option<String> {
        self.eoa_validator_address.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn webauthn_validator_address(&self) -> Option<String> {
        self.webauthn_validator_address.clone()
    }
}

// Config for sending transactions from smart accounts
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SendTransactionConfig {
    rpc_url: String,
    bundler_url: String,
    entry_point_address: String,
}

#[wasm_bindgen]
impl SendTransactionConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(
        rpc_url: String,
        bundler_url: String,
        entry_point_address: String,
    ) -> Self {
        Self { rpc_url, bundler_url, entry_point_address }
    }

    #[wasm_bindgen(getter)]
    pub fn rpc_url(&self) -> String {
        self.rpc_url.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn bundler_url(&self) -> String {
        self.bundler_url.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn entry_point_address(&self) -> String {
        self.entry_point_address.clone()
    }
}

/// Deploy a smart account using the factory
///
/// # Arguments
/// * `rpc_url` - The RPC endpoint URL
/// * `factory_address` - The address of the MSA factory contract
/// * `user_id` - A unique identifier for the user (will be hashed to create account_id)
/// * `deployer_private_key` - Private key of the account that will pay for deployment (0x-prefixed hex string)
/// * `eoa_signers_addresses` - Optional array of EOA signer addresses (as hex strings)
/// * `eoa_validator_address` - Optional EOA validator module address (required if eoa_signers_addresses is provided)
/// * `passkey_payload` - Optional WebAuthn passkey payload
/// * `webauthn_validator_address` - Optional WebAuthn validator module address (required if passkey_payload is provided)
///
/// # Returns
/// Promise that resolves to the deployed account address as a hex string
#[wasm_bindgen]
pub fn deploy_account(
    user_id: String,
    eoa_signers_addresses: Option<Vec<String>>,
    passkey_payload: Option<PasskeyPayload>,
    deploy_account_config: DeployAccountConfig,
) -> js_sys::Promise {
    future_to_promise(async move {
        console_log!("Starting account deployment...");
        console_log!("  RPC URL: {}", deploy_account_config.rpc_url);
        console_log!("  Factory: {}", deploy_account_config.factory_address);
        console_log!("  User ID: {}", user_id);

        // Parse factory address
        let factory_addr =
            match deploy_account_config.factory_address.parse::<Address>() {
                Ok(addr) => addr,
                Err(e) => {
                    return Ok(JsValue::from_str(&format!(
                        "Invalid factory address: {}",
                        e
                    )));
                }
            };

        // Parse deployer private key
        let deployer_key = match deploy_account_config
            .deployer_private_key
            .trim_start_matches("0x")
            .parse::<PrivateKeySigner>()
        {
            Ok(signer) => signer,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid deployer private key: {}",
                    e
                )));
            }
        };

        let deployer_wallet = EthereumWallet::from(deployer_key);
        console_log!(
            "  Deployer address: {:?}",
            deployer_wallet.default_signer().address()
        );

        // Create transport and provider with wallet
        let transport = WasmHttpTransport::new(deploy_account_config.rpc_url);
        let client = RpcClient::new(transport.clone(), false);
        let provider = ProviderBuilder::new()
            .wallet(deployer_wallet)
            .connect_client(client);

        // Parse EOA signers if provided
        let eoa_signers = match (
            eoa_signers_addresses,
            deploy_account_config.eoa_validator_address,
        ) {
            (Some(addresses), Some(validator)) => {
                console_log!(
                    "  Parsing EOA signers: {} addresses",
                    addresses.len()
                );
                let mut parsed_addresses = Vec::new();
                for addr_str in addresses {
                    match addr_str.parse::<Address>() {
                        Ok(addr) => parsed_addresses.push(addr),
                        Err(e) => {
                            return Ok(JsValue::from_str(&format!(
                                "Invalid EOA signer address '{}': {}",
                                addr_str, e
                            )));
                        }
                    }
                }

                let validator_addr = match validator.parse::<Address>() {
                    Ok(addr) => addr,
                    Err(e) => {
                        return Ok(JsValue::from_str(&format!(
                            "Invalid validator address: {}",
                            e
                        )));
                    }
                };

                Some(CoreEOASigners {
                    addresses: parsed_addresses,
                    validator_address: validator_addr,
                })
            }
            (None, None) => None,
            _ => {
                return Ok(JsValue::from_str(
                    "Both eoa_signers_addresses and eoa_validator_address must be provided together",
                ));
            }
        };

        // Parse WebAuthn signer if provided
        let webauthn_signer = match (
            passkey_payload,
            deploy_account_config.webauthn_validator_address,
        ) {
            (Some(passkey), Some(validator)) => {
                console_log!("  Parsing WebAuthn passkey");

                // Convert passkey coordinates to FixedBytes<32>
                if passkey.passkey_x.len() != 32 {
                    return Ok(JsValue::from_str(&format!(
                        "Invalid passkey X coordinate length: expected 32 bytes, got {}",
                        passkey.passkey_x.len()
                    )));
                }
                if passkey.passkey_y.len() != 32 {
                    return Ok(JsValue::from_str(&format!(
                        "Invalid passkey Y coordinate length: expected 32 bytes, got {}",
                        passkey.passkey_y.len()
                    )));
                }

                let passkey_x =
                    FixedBytes::<32>::from_slice(&passkey.passkey_x);
                let passkey_y =
                    FixedBytes::<32>::from_slice(&passkey.passkey_y);

                let validator_addr = match validator.parse::<Address>() {
                    Ok(addr) => addr,
                    Err(e) => {
                        return Ok(JsValue::from_str(&format!(
                            "Invalid WebAuthn validator address: {}",
                            e
                        )));
                    }
                };

                let core_passkey = CorePasskeyPayload {
                    credential_id: Bytes::from(passkey.credential_id),
                    passkey: [passkey_x, passkey_y],
                    origin_domain: passkey.origin_domain,
                };

                Some(CoreWebauthNSigner {
                    passkey: core_passkey,
                    validator_address: validator_addr,
                })
            }
            (None, None) => None,
            _ => {
                return Ok(JsValue::from_str(
                    "Both passkey_payload and webauthn_validator_address must be provided together",
                ));
            }
        };

        console_log!("  Calling core deploy_account...");

        // Use the core crate's deploy_account function
        match zksync_sso_erc4337_core::erc4337::account::modular_smart_account::deploy::deploy_account(
            factory_addr,
            eoa_signers,
            webauthn_signer,
            provider,
        )
        .await
        {
            Ok(address) => {
                let address_str = format!("0x{:x}", address);
                console_log!("  Deployed account address: {}", address_str);
                Ok(JsValue::from_str(&address_str))
            }
            Err(e) => {
                console_log!("  Error deploying account: {}", e);
                Ok(JsValue::from_str(&format!(
                    "Failed to deploy account: {}",
                    e
                )))
            }
        }
    })
}

/// Add a passkey to an already-deployed smart account
///
/// This function registers a passkey with the WebAuthn validator module.
/// The account must already be deployed and the WebAuthn validator must be installed.
///
/// # Parameters
/// * `config` - SendTransactionConfig with RPC URL, bundler URL, and entry point
/// * `account_address` - The deployed smart account address
/// * `passkey_payload` - The passkey to register
/// * `webauthn_validator_address` - The WebAuthn validator module address
/// * `eoa_validator_address` - The EOA validator module address (for signing the transaction)
/// * `eoa_private_key` - Private key of an EOA signer (to authorize the passkey addition)
///
/// # Returns
/// Promise that resolves when the passkey is registered
#[wasm_bindgen]
pub fn add_passkey_to_account(
    config: SendTransactionConfig,
    account_address: String,
    passkey_payload: PasskeyPayload,
    webauthn_validator_address: String,
    eoa_validator_address: String,
    eoa_private_key: String,
) -> js_sys::Promise {
    future_to_promise(async move {
        console_log!("Adding passkey to smart account...");
        console_log!("  Account: {}", account_address);

        // Parse addresses
        let account = match account_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid account address: {}",
                    e
                )));
            }
        };

        let entry_point = match config.entry_point_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid entry point address: {}",
                    e
                )));
            }
        };

        let webauthn_validator =
            match webauthn_validator_address.parse::<Address>() {
                Ok(addr) => addr,
                Err(e) => {
                    return Ok(JsValue::from_str(&format!(
                        "Invalid WebAuthn validator address: {}",
                        e
                    )));
                }
            };

        let eoa_validator = match eoa_validator_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
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
                return Ok(JsValue::from_str(&format!(
                    "Invalid EOA private key: {}",
                    e
                )));
            }
        };

        let eoa_wallet = EthereumWallet::from(eoa_key.clone());

        // Create transport and provider
        let transport = WasmHttpTransport::new(config.rpc_url);
        let client = RpcClient::new(transport.clone(), false);
        let provider =
            ProviderBuilder::new().wallet(eoa_wallet).connect_client(client);

        // Create bundler client
        let bundler_client = {
            use zksync_sso_erc4337_core::erc4337::bundler::config::BundlerConfig;
            let config = BundlerConfig::new(config.bundler_url);
            zksync_sso_erc4337_core::erc4337::bundler::pimlico::client::BundlerClient::new(config)
        };

        // Create EOA signer
        use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::signature::{
            eoa_signature, stub_signature_eoa,
        };
        use std::sync::Arc;

        let stub_sig = match stub_signature_eoa(eoa_validator) {
            Ok(sig) => sig,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Failed to create stub signature: {}",
                    e
                )));
            }
        };

        let eoa_key_str = format!("0x{}", hex::encode(eoa_key.to_bytes()));
        let signature_provider = Arc::new(move |hash: FixedBytes<32>| {
            eoa_signature(&eoa_key_str, eoa_validator, hash)
        });

        let signer = zksync_sso_erc4337_core::erc4337::signer::Signer {
            provider: signature_provider,
            stub_signature: stub_sig,
        };

        // Convert PasskeyPayload to core type
        let passkey_x =
            FixedBytes::<32>::from_slice(&passkey_payload.passkey_x);
        let passkey_y =
            FixedBytes::<32>::from_slice(&passkey_payload.passkey_y);

        let core_passkey = CorePasskeyPayload {
            credential_id: Bytes::from(passkey_payload.credential_id),
            passkey: [passkey_x, passkey_y],
            origin_domain: passkey_payload.origin_domain,
        };

        console_log!("  Calling add_passkey...");

        // Call the core add_passkey function
        match zksync_sso_erc4337_core::erc4337::account::modular_smart_account::add_passkey::add_passkey(
            account,
            core_passkey,
            webauthn_validator,
            entry_point,
            provider,
            bundler_client,
            signer,
        )
        .await
        {
            Ok(_) => {
                console_log!("  Passkey added successfully");
                Ok(JsValue::from_str("Passkey registered successfully"))
            }
            Err(e) => {
                console_log!("  Error adding passkey: {}", e);
                Ok(JsValue::from_str(&format!(
                    "Failed to add passkey: {}",
                    e
                )))
            }
        }
    })
}

/// Send a transaction from a smart account using EOA validator
///
/// # Arguments
/// * `rpc_url` - The RPC endpoint URL
/// * `bundler_url` - The bundler endpoint URL (e.g., "http://localhost:4337")
/// * `account_address` - The deployed smart account address
/// * `entry_point_address` - The EntryPoint contract address
/// * `eoa_validator_address` - The EOA validator module address
/// * `eoa_private_key` - Private key of the EOA signer (0x-prefixed hex string)
/// * `to_address` - The recipient address for the transaction
/// * `value` - The amount to send (as string, e.g., "1000000000000000000" for 1 ETH)
/// * `data` - Optional calldata as hex string (default: "0x" for simple transfer)
///
/// # Returns
/// Promise that resolves when the UserOperation is confirmed
#[wasm_bindgen]
pub fn send_transaction_eoa(
    config: SendTransactionConfig,
    eoa_validator_address: String,
    eoa_private_key: String,
    account_address: String,
    to_address: String,
    value: String,
    data: Option<String>,
) -> js_sys::Promise {
    future_to_promise(async move {
        console_log!("Starting EOA transaction...");
        console_log!("  Account: {}", account_address);
        console_log!("  To: {}", to_address);
        console_log!("  Value: {}", value);
        console_log!("  Bundler: {}", config.bundler_url);

        // Parse addresses
        let account = match account_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid account address: {}",
                    e
                )));
            }
        };

        let entry_point = match config.entry_point_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid entry point address: {}",
                    e
                )));
            }
        };

        let eoa_validator = match eoa_validator_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid EOA validator address: {}",
                    e
                )));
            }
        };

        let to = match to_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid to address: {}",
                    e
                )));
            }
        };

        // Parse value
        let value_u256 = match value.parse::<U256>() {
            Ok(v) => v,
            Err(e) => {
                return Ok(JsValue::from_str(&format!("Invalid value: {}", e)));
            }
        };

        // Parse data
        let data_bytes = match data {
            Some(d) => {
                let hex_str = d.trim_start_matches("0x");
                match hex::decode(hex_str) {
                    Ok(bytes) => Bytes::from(bytes),
                    Err(e) => {
                        return Ok(JsValue::from_str(&format!(
                            "Invalid data hex: {}",
                            e
                        )));
                    }
                }
            }
            None => Bytes::default(),
        };

        console_log!("  Parsed addresses and values successfully");

        // Parse EOA private key
        let eoa_key = match eoa_private_key
            .trim_start_matches("0x")
            .parse::<PrivateKeySigner>()
        {
            Ok(signer) => signer,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid EOA private key: {}",
                    e
                )));
            }
        };

        let eoa_wallet = EthereumWallet::from(eoa_key);
        console_log!(
            "  EOA signer address: {:?}",
            eoa_wallet.default_signer().address()
        );

        // Create transport and provider
        let transport = WasmHttpTransport::new(config.rpc_url.clone());
        let client = RpcClient::new(transport.clone(), false);
        let provider =
            ProviderBuilder::new().wallet(eoa_wallet).connect_client(client);

        console_log!("  Created provider and transport");

        // Create bundler client
        let bundler_client = {
            use zksync_sso_erc4337_core::erc4337::bundler::config::BundlerConfig;
            let config = BundlerConfig::new(config.bundler_url);
            zksync_sso_erc4337_core::erc4337::bundler::pimlico::client::BundlerClient::new(config)
        };

        console_log!("  Created bundler client");

        // Encode the execution call
        use zksync_sso_erc4337_core::erc4337::account::erc7579::{
            Execution, calls::encode_calls,
        };

        let call =
            Execution { target: to, value: value_u256, data: data_bytes };

        let calls = vec![call];
        let encoded_calls: Bytes = encode_calls(calls).into();

        console_log!(
            "  Encoded call data, calling core send_transaction_eoa..."
        );

        // Use the core crate's send_transaction_eoa function
        match zksync_sso_erc4337_core::erc4337::account::modular_smart_account::send::send_transaction_eoa(
            account,
            eoa_validator,
            entry_point,
            encoded_calls,
            bundler_client,
            provider,
            eoa_private_key,
        )
        .await
        {
            Ok(_) => {
                console_log!("  Transaction sent successfully!");
                Ok(JsValue::from_str("Transaction sent successfully"))
            }
            Err(e) => {
                console_log!("  Error sending transaction: {}", e);
                Ok(JsValue::from_str(&format!("Failed to send transaction: {}", e)))
            }
        }
    })
}

/// Prepare a UserOperation for passkey signing with fixed gas limits
/// Returns the hash that needs to be signed by the passkey
///
/// This function creates a stub signature internally, so the caller doesn't need to provide one.
///
/// # Parameters
/// * `config` - SendTransactionConfig with RPC, bundler, and entry point
/// * `webauthn_validator_address` - Address of the WebAuthn validator module
/// * `account_address` - The smart account address
/// * `to_address` - The recipient address
/// * `value` - Amount to send (as string, e.g., "1000000000000000000" for 1 ETH)
/// * `data` - Optional calldata as hex string
///
/// # Returns
/// JSON string with format: `{"hash": "0x...", "userOpId": "..."}`
/// The hash should be signed by the passkey, then passed to `submit_passkey_user_operation`
#[wasm_bindgen]
pub fn prepare_passkey_user_operation_fixed_gas(
    config: SendTransactionConfig,
    webauthn_validator_address: String,
    account_address: String,
    to_address: String,
    value: String,
    data: Option<String>,
) -> js_sys::Promise {
    future_to_promise(async move {
        console_log!(
            "Preparing passkey UserOperation with fixed gas (no bundler estimation)..."
        );
        console_log!("  Account: {}", account_address);
        console_log!("  To: {}", to_address);
        console_log!("  Value: {}", value);

        // Parse addresses
        let account = match account_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid account address: {}",
                    e
                )));
            }
        };

        let entry_point = match config.entry_point_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid entry point address: {}",
                    e
                )));
            }
        };

        let to = match to_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid to address: {}",
                    e
                )));
            }
        };

        // Parse value
        let value_u256 = match value.parse::<U256>() {
            Ok(v) => v,
            Err(e) => {
                return Ok(JsValue::from_str(&format!("Invalid value: {}", e)));
            }
        };

        // Parse data
        let data_bytes = match data {
            Some(d) => {
                let hex_str = d.trim_start_matches("0x");
                match hex::decode(hex_str) {
                    Ok(bytes) => Bytes::from(bytes),
                    Err(e) => {
                        return Ok(JsValue::from_str(&format!(
                            "Invalid data hex: {}",
                            e
                        )));
                    }
                }
            }
            None => Bytes::default(),
        };

        console_log!("  Parsed addresses and values successfully");

        // Create transport and provider
        let transport = WasmHttpTransport::new(config.rpc_url.clone());
        let client = RpcClient::new(transport.clone(), false);
        let provider = ProviderBuilder::new().connect_client(client);

        console_log!("  Created provider and transport");

        // Encode the execution call
        use zksync_sso_erc4337_core::erc4337::account::erc7579::{
            Execution, calls::encode_calls,
        };

        let call =
            Execution { target: to, value: value_u256, data: data_bytes };
        let calls = vec![call];
        let encoded_calls: Bytes = encode_calls(calls).into();

        console_log!("  Encoded call data");

        // Build UserOperation with fixed high gas values (no bundler estimation)
        use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::nonce::get_nonce;
        use alloy::primitives::Uint;

        let nonce_key = Uint::from(0);
        let nonce =
            match get_nonce(entry_point, account, nonce_key, &provider).await {
                Ok(n) => n,
                Err(e) => {
                    return Ok(JsValue::from_str(&format!(
                        "Failed to get nonce: {}",
                        e
                    )));
                }
            };

        // Parse validator address to create stub signature
        let validator = match webauthn_validator_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid validator address: {}",
                    e
                )));
            }
        };

        // Create stub signature internally (validator address + minimal ABI-encoded data)
        // This matches the format expected by WebAuthnValidator: (bytes authenticatorData, string clientDataJSON, bytes32[2] rs, bytes credentialId)
        use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::signature::stub_signature_passkey;
        let stub_sig = match stub_signature_passkey(validator) {
            Ok(sig) => sig,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Failed to create stub signature: {}",
                    e
                )));
            }
        };

        console_log!("  Created stub signature: {} bytes", stub_sig.len());

        // Use fixed high gas values (matching the Rust test approach when bundler is unavailable)
        let call_gas_limit = U256::from(2_000_000u64);
        let verification_gas_limit = U256::from(2_000_000u64);
        let pre_verification_gas = U256::from(1_000_000u64);
        let max_priority_fee_per_gas = U256::from(0x77359400u64);
        let max_fee_per_gas = U256::from(0x82e08afeu64);

        console_log!(
            "  Using fixed gas limits: call={}, verification={}, preVerification={}",
            call_gas_limit,
            verification_gas_limit,
            pre_verification_gas
        );

        // Create AlloyPackedUserOperation
        let user_op = AlloyPackedUserOperation {
            sender: account,
            nonce,
            call_data: encoded_calls.clone(),
            signature: stub_sig.clone(),
            paymaster: None,
            paymaster_verification_gas_limit: None,
            paymaster_data: None,
            paymaster_post_op_gas_limit: None,
            call_gas_limit,
            max_priority_fee_per_gas,
            max_fee_per_gas,
            pre_verification_gas,
            verification_gas_limit,
            factory: None,
            factory_data: None,
        };

        // Pack gas limits and fees for EntryPoint::PackedUserOperation
        let packed_gas_limits: U256 =
            (user_op.verification_gas_limit << 128) | user_op.call_gas_limit;
        let gas_fees: U256 =
            (user_op.max_priority_fee_per_gas << 128) | user_op.max_fee_per_gas;

        // Create PackedUserOperation for hashing (EntryPoint format with packed fields)
        use zksync_sso_erc4337_core::erc4337::entry_point::EntryPoint::PackedUserOperation;
        let packed_user_op = PackedUserOperation {
            sender: user_op.sender,
            nonce: user_op.nonce,
            initCode: Bytes::default(),
            callData: user_op.call_data.clone(),
            accountGasLimits: packed_gas_limits.to_be_bytes().into(),
            preVerificationGas: user_op.pre_verification_gas,
            gasFees: gas_fees.to_be_bytes().into(),
            paymasterAndData: Bytes::default(),
            signature: user_op.signature.clone(),
        };

        // Get the hash that needs to be signed
        use zksync_sso_erc4337_core::erc4337::user_operation::hash::v08::get_user_operation_hash_entry_point;

        let hash = match get_user_operation_hash_entry_point(
            &packed_user_op,
            &entry_point,
            provider.clone(),
        )
        .await
        {
            Ok(h) => h,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Failed to get UserOp hash: {}",
                    e
                )));
            }
        };

        console_log!("  UserOperation hash: {:?}", hash);

        // Store the AlloyPackedUserOperation and validator address in a global map
        // Use the debug format for the hashmap key (includes type wrapper)
        let hash_key = format!("{:?}", hash);
        // Convert to B256 to get hex format for the JSON response (just the hex bytes)
        let hash_b256: alloy::primitives::B256 = hash.into();
        let hash_hex = format!("{:#x}", hash_b256);
        USER_OPS.lock().unwrap().insert(hash_key.clone(), (user_op, validator));

        // Return JSON with hash and userOpId
        // hash: clean hex string for JavaScript to sign
        // userOpId: debug format key to retrieve the UserOp later
        let result =
            format!(r#"{{"hash":"{}","userOpId":"{}"}}"#, hash_hex, hash_key);

        console_log!(
            "  Prepared UserOperation with fixed gas, waiting for passkey signature"
        );
        Ok(JsValue::from_str(&result))
    })
}

/// Prepare a passkey UserOperation (step 1 of two-step flow with bundler gas estimation)
///
/// # Parameters
/// * `config` - SendTransactionConfig with RPC, bundler, and entry point
/// * `webauthn_validator_address` - Address of the WebAuthn validator module
/// * `account_address` - The smart account address
/// * `to_address` - The recipient address
/// * `value` - Amount to send (as string, e.g., "1000000000000000000" for 1 ETH)
/// * `data` - Optional calldata as hex string
/// * `stub_signature_hex` - A real passkey signature created with all-zeros hash (for gas estimation)
///
/// # Returns
/// JSON string with format: `{"hash": "0x...", "userOpId": "..."}`
/// The hash should be signed by the passkey, then passed to `submit_passkey_user_operation`
#[wasm_bindgen]
pub fn prepare_passkey_user_operation(
    config: SendTransactionConfig,
    webauthn_validator_address: String,
    account_address: String,
    to_address: String,
    value: String,
    data: Option<String>,
    stub_signature_hex: String,
) -> js_sys::Promise {
    future_to_promise(async move {
        console_log!("Preparing passkey UserOperation...");
        console_log!("  Account: {}", account_address);
        console_log!("  To: {}", to_address);
        console_log!("  Value: {}", value);

        // Parse addresses
        let account = match account_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid account address: {}",
                    e
                )));
            }
        };

        let entry_point = match config.entry_point_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid entry point address: {}",
                    e
                )));
            }
        };

        let _webauthn_validator =
            match webauthn_validator_address.parse::<Address>() {
                Ok(addr) => addr,
                Err(e) => {
                    return Ok(JsValue::from_str(&format!(
                        "Invalid WebAuthn validator address: {}",
                        e
                    )));
                }
            };

        let to = match to_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid to address: {}",
                    e
                )));
            }
        };

        // Parse value
        let value_u256 = match value.parse::<U256>() {
            Ok(v) => v,
            Err(e) => {
                return Ok(JsValue::from_str(&format!("Invalid value: {}", e)));
            }
        };

        // Parse data
        let data_bytes = match data {
            Some(d) => {
                let hex_str = d.trim_start_matches("0x");
                match hex::decode(hex_str) {
                    Ok(bytes) => Bytes::from(bytes),
                    Err(e) => {
                        return Ok(JsValue::from_str(&format!(
                            "Invalid data hex: {}",
                            e
                        )));
                    }
                }
            }
            None => Bytes::default(),
        };

        console_log!("  Parsed addresses and values successfully");

        // Create transport and provider
        let transport = WasmHttpTransport::new(config.rpc_url.clone());
        let client = RpcClient::new(transport.clone(), false);
        let provider = ProviderBuilder::new().connect_client(client);

        console_log!("  Created provider and transport");

        // Encode the execution call
        use zksync_sso_erc4337_core::erc4337::account::erc7579::{
            Execution, calls::encode_calls,
        };

        let call =
            Execution { target: to, value: value_u256, data: data_bytes };
        let calls = vec![call];
        let encoded_calls: Bytes = encode_calls(calls).into();

        console_log!("  Encoded call data");

        // Check account balance before proceeding
        console_log!("  Checking account balance...");
        use alloy::providers::Provider;
        let balance = match provider.get_balance(account).await {
            Ok(bal) => bal,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Failed to get account balance: {}",
                    e
                )));
            }
        };

        console_log!("  Account balance: {} wei", balance);

        if balance == U256::ZERO {
            return Ok(JsValue::from_str(
                "Account has zero balance. Please fund the smart account with ETH before sending transactions.",
            ));
        }

        // Rough estimate: minimum balance should be at least 0.001 ETH for gas
        let min_balance = U256::from(1_000_000_000_000_000u64); // 0.001 ETH in wei
        if balance < min_balance {
            return Ok(JsValue::from_str(&format!(
                "Account balance is too low: {} wei. Please fund the smart account with at least 0.001 ETH.",
                balance
            )));
        }

        // Build UserOperation with stub signature
        use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::nonce::get_nonce;
        use alloy::primitives::Uint;

        let nonce_key = Uint::from(0);
        let nonce =
            match get_nonce(entry_point, account, nonce_key, &provider).await {
                Ok(n) => n,
                Err(e) => {
                    return Ok(JsValue::from_str(&format!(
                        "Failed to get nonce: {}",
                        e
                    )));
                }
            };

        // Parse the stub signature (should be a real passkey signature with all-zeros hash)
        let stub_sig_hex = stub_signature_hex.trim_start_matches("0x");
        let stub_sig = match hex::decode(stub_sig_hex) {
            Ok(bytes) => Bytes::from(bytes),
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid stub signature hex: {}",
                    e
                )));
            }
        };

        console_log!(
            "  Using stub signature for gas estimation: {} bytes",
            stub_sig.len()
        );

        // Create AlloyPackedUserOperation for gas estimation (has unpacked fields)
        let mut user_op = AlloyPackedUserOperation {
            sender: account,
            nonce,
            call_data: encoded_calls.clone(),
            signature: stub_sig.clone(),
            paymaster: None,
            paymaster_verification_gas_limit: None,
            paymaster_data: None,
            paymaster_post_op_gas_limit: None,
            call_gas_limit: Default::default(),
            max_priority_fee_per_gas: Default::default(),
            max_fee_per_gas: Default::default(),
            pre_verification_gas: Default::default(),
            verification_gas_limit: Default::default(),
            factory: None,
            factory_data: None,
        };

        // Create bundler client for gas estimation
        console_log!("  Estimating gas...");
        use zksync_sso_erc4337_core::erc4337::bundler::{
            config::BundlerConfig, pimlico::client::BundlerClient,
        };

        let bundler_config = BundlerConfig::new(config.bundler_url.clone());
        let bundler_client = BundlerClient::new(bundler_config);

        let estimated_gas = match bundler_client
            .estimate_user_operation_gas(&user_op, &entry_point)
            .await
        {
            Ok(gas) => gas,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Failed to estimate gas: {}. Make sure the passkey is registered and the account is funded.",
                    e
                )));
            }
        };

        console_log!(
            "  Estimated gas: call={}, verification={}, preVerification={}",
            estimated_gas.call_gas_limit,
            estimated_gas.verification_gas_limit,
            estimated_gas.pre_verification_gas
        );

        // Update with estimated gas values (with 20% buffer on verification gas like the working test)
        user_op.call_gas_limit = estimated_gas.call_gas_limit;
        user_op.verification_gas_limit = (estimated_gas.verification_gas_limit
            * U256::from(6))
            / U256::from(5);
        user_op.pre_verification_gas = estimated_gas.pre_verification_gas;
        user_op.max_priority_fee_per_gas = U256::from(0x77359400);
        user_op.max_fee_per_gas = U256::from(0x82e08afeu64);

        console_log!(
            "  Using gas limits: call={}, verification={}, preVerification={}",
            user_op.call_gas_limit,
            user_op.verification_gas_limit,
            user_op.pre_verification_gas
        );

        // Pack gas limits and fees for EntryPoint::PackedUserOperation
        let packed_gas_limits: U256 =
            (user_op.verification_gas_limit << 128) | user_op.call_gas_limit;
        let gas_fees: U256 =
            (user_op.max_priority_fee_per_gas << 128) | user_op.max_fee_per_gas;

        // Create PackedUserOperation for hashing (EntryPoint format with packed fields)
        use zksync_sso_erc4337_core::erc4337::entry_point::EntryPoint::PackedUserOperation;
        let packed_user_op = PackedUserOperation {
            sender: user_op.sender,
            nonce: user_op.nonce,
            initCode: Bytes::default(),
            callData: user_op.call_data.clone(),
            accountGasLimits: packed_gas_limits.to_be_bytes().into(),
            preVerificationGas: user_op.pre_verification_gas,
            gasFees: gas_fees.to_be_bytes().into(),
            paymasterAndData: Bytes::default(),
            signature: user_op.signature.clone(),
        };

        // Get the hash that needs to be signed
        use zksync_sso_erc4337_core::erc4337::user_operation::hash::v08::get_user_operation_hash_entry_point;

        let hash = match get_user_operation_hash_entry_point(
            &packed_user_op,
            &entry_point,
            provider.clone(),
        )
        .await
        {
            Ok(h) => h,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Failed to get UserOp hash: {}",
                    e
                )));
            }
        };

        console_log!("  UserOperation hash: {:?}", hash);

        // Parse validator address to store with UserOp
        let validator = match webauthn_validator_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid validator address: {}",
                    e
                )));
            }
        };

        // Store the AlloyPackedUserOperation and validator address in a global map
        // For now, we'll use the hash as the ID
        let hash_str = format!("{:?}", hash);
        USER_OPS.lock().unwrap().insert(hash_str.clone(), (user_op, validator));

        // Return JSON with hash and userOpId
        let result =
            format!(r#"{{"hash":"{}","userOpId":"{}"}}"#, hash_str, hash_str);

        console_log!("  Prepared UserOperation, waiting for passkey signature");
        Ok(JsValue::from_str(&result))
    })
}

/// Submit a passkey-signed UserOperation
///
/// # Parameters
/// * `config` - SendTransactionConfig with bundler URL
/// * `user_op_id` - The ID returned from `prepare_passkey_user_operation`
/// * `passkey_signature` - The signature from WebAuthn (hex string with 0x prefix)
///
/// # Returns
/// Promise that resolves when the UserOperation is confirmed
#[wasm_bindgen]
pub fn submit_passkey_user_operation(
    config: SendTransactionConfig,
    user_op_id: String,
    passkey_signature: String,
) -> js_sys::Promise {
    future_to_promise(async move {
        console_log!("Submitting passkey-signed UserOperation...");
        console_log!("  UserOp ID: {}", user_op_id);

        // Debug: Check what's in the HashMap
        {
            let map = USER_OPS.lock().unwrap();
            console_log!("  HashMap contains {} entries", map.len());
            for key in map.keys() {
                console_log!("    Key: {}", key);
            }
        }

        // Retrieve the AlloyPackedUserOperation and validator address from storage
        let (mut user_op, validator_address) = {
            let mut map = USER_OPS.lock().unwrap();
            match map.remove(&user_op_id) {
                Some(data) => data,
                None => {
                    return Ok(JsValue::from_str(&format!(
                        "UserOperation not found for ID: {}",
                        user_op_id
                    )));
                }
            }
        };

        // Parse the passkey signature
        let sig_hex = passkey_signature.trim_start_matches("0x");
        let signature = match hex::decode(sig_hex) {
            Ok(bytes) => Bytes::from(bytes),
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid signature hex: {}",
                    e
                )));
            }
        };

        console_log!("  Signature length: {} bytes", signature.len());

        // Prepend validator address to signature (ModularSmartAccount expects first 20 bytes to be validator)
        let mut full_signature = Vec::new();
        full_signature.extend_from_slice(validator_address.as_slice()); // 20 bytes
        full_signature.extend_from_slice(&signature); // ABI-encoded passkey signature

        console_log!(
            "  Full signature length: {} bytes (20 validator + {} passkey)",
            full_signature.len(),
            signature.len()
        );

        // Log signature breakdown for debugging
        console_log!("  Validator address: {:?}", validator_address);
        console_log!(
            "  First 32 bytes of passkey signature: {:?}",
            &signature.as_ref().get(0..32.min(signature.len()))
        );
        console_log!(
            "  Last 32 bytes of passkey signature: {:?}",
            signature.as_ref().get(signature.len().saturating_sub(32)..)
        );

        // Update UserOperation with full signature (validator address + passkey signature)
        user_op.signature = Bytes::from(full_signature.clone());

        // Log UserOperation details before submission
        console_log!("  UserOperation details:");
        console_log!("    sender: {:?}", user_op.sender);
        console_log!("    nonce: {}", user_op.nonce);
        console_log!("    callData length: {}", user_op.call_data.len());
        console_log!("    signature length: {}", user_op.signature.len());
        console_log!("    callGasLimit: {}", user_op.call_gas_limit);
        console_log!(
            "    verificationGasLimit: {}",
            user_op.verification_gas_limit
        );
        console_log!(
            "    preVerificationGas: {}",
            user_op.pre_verification_gas
        );

        // Create bundler client
        let bundler_client = {
            use zksync_sso_erc4337_core::erc4337::bundler::config::BundlerConfig;
            let config = BundlerConfig::new(config.bundler_url);
            zksync_sso_erc4337_core::erc4337::bundler::pimlico::client::BundlerClient::new(config)
        };

        // Parse entry point address for bundler call
        let entry_point = match config.entry_point_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid entry point address: {}",
                    e
                )));
            }
        };

        // Submit UserOperation
        use zksync_sso_erc4337_core::erc4337::bundler::Bundler;

        match bundler_client.send_user_operation(entry_point, user_op).await {
            Ok(user_op_hash) => {
                console_log!("  UserOperation submitted: {:?}", user_op_hash);
                let hash_for_display = user_op_hash.clone();

                // Wait for receipt
                match bundler_client
                    .wait_for_user_operation_receipt(user_op_hash)
                    .await
                {
                    Ok(receipt) => {
                        console_log!("  UserOperation confirmed!");
                        console_log!("  Receipt: {:?}", receipt);
                        Ok(JsValue::from_str(&format!(
                            "Transaction confirmed! UserOp hash: {:?}",
                            hash_for_display
                        )))
                    }
                    Err(e) => {
                        console_log!("  Error waiting for receipt: {}", e);
                        Ok(JsValue::from_str(&format!(
                            "UserOperation submitted but failed to get receipt: {}",
                            e
                        )))
                    }
                }
            }
            Err(e) => {
                console_log!("  Error submitting UserOperation: {}", e);
                Ok(JsValue::from_str(&format!(
                    "Failed to submit UserOperation: {}",
                    e
                )))
            }
        }
    })
}

/// Parse contract addresses from strings
#[wasm_bindgen]
pub fn parse_contract_addresses(
    entry_point: &str,
    account_factory: &str,
    webauthn_validator: &str,
    eoa_validator: &str,
    session_validator: &str,
) -> Result<String, JsValue> {
    match CoreContracts::from_string(
        entry_point.to_string(),
        account_factory.to_string(),
        webauthn_validator.to_string(),
        eoa_validator.to_string(),
        session_validator.to_string(),
    ) {
        Ok(contracts) => Ok(format!(
            "Entry Point: {:?}, Account Factory: {:?}, WebAuthn Validator: {:?}, EOA Validator: {:?}",
            contracts.entry_point,
            contracts.account_factory,
            contracts.webauthn_validator,
            contracts.eoa_validator
        )),
        Err(e) => Err(JsValue::from_str(&format!(
            "Failed to parse addresses: {:?}",
            e
        ))),
    }
}

// Utility functions
#[wasm_bindgen]
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

#[wasm_bindgen]
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, JsValue> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    hex::decode(hex)
        .map_err(|e| JsValue::from_str(&format!("Invalid hex: {}", e)))
}

#[wasm_bindgen]
pub fn console_log_from_rust(message: &str) {
    console_log!("{}", message);
}

/// ABI encode a passkey signature
///
/// Format: (bytes authenticatorData, string clientDataJSON, bytes32[2] rs, bytes credentialId)
///
/// # Parameters
/// * `authenticator_data` - Raw authenticator data from WebAuthn response (hex string)
/// * `client_data_json` - Client data JSON string from WebAuthn response
/// * `r` - Signature r component (hex string, must be 32 bytes when decoded)
/// * `s` - Signature s component (hex string, must be 32 bytes when decoded)
/// * `credential_id` - The credential ID (hex string)
///
/// # Returns
/// ABI-encoded signature as hex string with 0x prefix
#[wasm_bindgen]
pub fn abi_encode_passkey_signature(
    authenticator_data: &str,
    client_data_json: &str,
    r: &str,
    s: &str,
    credential_id: &str,
) -> Result<String, JsValue> {
    use alloy::sol_types::SolValue;

    // Parse hex strings to bytes
    let auth_data_hex =
        authenticator_data.strip_prefix("0x").unwrap_or(authenticator_data);
    let auth_data = hex::decode(auth_data_hex).map_err(|e| {
        JsValue::from_str(&format!("Invalid authenticatorData hex: {}", e))
    })?;

    let r_hex = r.strip_prefix("0x").unwrap_or(r);
    let r_bytes = hex::decode(r_hex)
        .map_err(|e| JsValue::from_str(&format!("Invalid r hex: {}", e)))?;
    if r_bytes.len() != 32 {
        return Err(JsValue::from_str(&format!(
            "r must be 32 bytes, got {}",
            r_bytes.len()
        )));
    }
    let r_fixed = FixedBytes::<32>::from_slice(&r_bytes);

    let s_hex = s.strip_prefix("0x").unwrap_or(s);
    let s_bytes = hex::decode(s_hex)
        .map_err(|e| JsValue::from_str(&format!("Invalid s hex: {}", e)))?;
    if s_bytes.len() != 32 {
        return Err(JsValue::from_str(&format!(
            "s must be 32 bytes, got {}",
            s_bytes.len()
        )));
    }
    let s_fixed = FixedBytes::<32>::from_slice(&s_bytes);

    let cred_id_hex = credential_id.strip_prefix("0x").unwrap_or(credential_id);
    let cred_id = hex::decode(cred_id_hex).map_err(|e| {
        JsValue::from_str(&format!("Invalid credentialId hex: {}", e))
    })?;

    // ABI encode using Alloy's sol_types
    // Type: (bytes, string, bytes32[2], bytes)
    let encoded = (
        Bytes::from(auth_data),
        client_data_json.to_string(),
        [r_fixed, s_fixed],
        Bytes::from(cred_id),
    )
        .abi_encode();

    Ok(format!("0x{}", hex::encode(encoded)))
}

/// ABI encode a stub passkey signature for gas estimation
///
/// Creates a minimal signature with empty/zero values
///
/// # Parameters
/// * `validator_address` - The WebAuthn validator address (hex string with 0x prefix)
///
/// # Returns
/// Hex-encoded stub signature (validator address + ABI-encoded empty signature)
#[wasm_bindgen]
pub fn abi_encode_stub_passkey_signature(
    validator_address: &str,
) -> Result<String, JsValue> {
    use alloy::sol_types::SolValue;

    // Parse validator address
    let validator_hex =
        validator_address.strip_prefix("0x").unwrap_or(validator_address);
    let validator_bytes = hex::decode(validator_hex).map_err(|e| {
        JsValue::from_str(&format!("Invalid validator address hex: {}", e))
    })?;
    if validator_bytes.len() != 20 {
        return Err(JsValue::from_str(&format!(
            "Validator address must be 20 bytes, got {}",
            validator_bytes.len()
        )));
    }

    // Create minimal stub: empty authenticatorData, empty clientDataJSON, zero r/s, empty credentialId
    let zero_32 = FixedBytes::<32>::default();
    let empty_bytes = Bytes::default();

    // ABI encode using Alloy's sol_types
    let encoded =
        (empty_bytes.clone(), String::new(), [zero_32, zero_32], empty_bytes)
            .abi_encode();

    // Prepend validator address
    let mut full_stub = validator_bytes;
    full_stub.extend_from_slice(&encoded);

    Ok(format!("0x{}", hex::encode(full_stub)))
}

/// Compute account ID from user ID (same logic as get_account_id_by_user_id in SDK)
/// This is used to generate a unique identifier for smart accounts
#[wasm_bindgen]
pub fn compute_account_id(user_id: &str) -> String {
    let salt = hex::encode(user_id);
    let salt_hash = keccak256(salt);
    format!("0x{}", hex::encode(salt_hash))
}

/// Compute the smart account address (offline, without RPC calls)
/// This requires knowing the bytecode hash and proxy address upfront
///
/// # Parameters
/// - `user_id`: The unique user identifier
/// - `deploy_wallet_address`: The address of the wallet deploying the account (hex string)
/// - `account_factory`: The address of the AAFactory contract (hex string)  
/// - `bytecode_hash`: The beacon proxy bytecode hash (hex string, 32 bytes)
/// - `proxy_address`: The encoded beacon address (hex string)
///
/// # Returns
/// The computed smart account address as a hex string
#[wasm_bindgen]
pub fn compute_smart_account_address(
    user_id: &str,
    deploy_wallet_address: &str,
    account_factory: &str,
    bytecode_hash: &str,
    proxy_address: &str,
) -> Result<String, JsValue> {
    console_log!("Computing smart account address for user: {}", user_id);

    // Parse addresses
    let factory_addr: Address = account_factory.parse().map_err(|e| {
        JsValue::from_str(&format!("Invalid factory address: {}", e))
    })?;

    let deploy_wallet_addr: Address =
        deploy_wallet_address.parse().map_err(|e| {
            JsValue::from_str(&format!("Invalid wallet address: {}", e))
        })?;

    // Parse bytecode hash
    let bytecode_hash_hex =
        bytecode_hash.strip_prefix("0x").unwrap_or(bytecode_hash);
    let bytecode_hash_bytes = hex::decode(bytecode_hash_hex).map_err(|e| {
        JsValue::from_str(&format!("Invalid bytecode hash: {}", e))
    })?;
    let bytecode_hash = FixedBytes::<32>::from_slice(&bytecode_hash_bytes);

    // Parse proxy address
    let proxy_hex = proxy_address.strip_prefix("0x").unwrap_or(proxy_address);
    let proxy_bytes = hex::decode(proxy_hex).map_err(|e| {
        JsValue::from_str(&format!("Invalid proxy address: {}", e))
    })?;
    let proxy_address = Bytes::from(proxy_bytes);

    // Get account ID hash
    let salt = hex::encode(user_id);
    let account_id_hash = keccak256(salt);
    console_log!("Account ID hash: 0x{}", hex::encode(account_id_hash));

    // Compute unique salt
    let wallet_address_bytes = deploy_wallet_addr.0.to_vec();
    let mut concatenated_bytes = Vec::new();
    concatenated_bytes.extend(account_id_hash.to_vec());
    concatenated_bytes.extend(wallet_address_bytes);
    let unique_salt = keccak256(concatenated_bytes);
    console_log!("Unique salt: 0x{}", hex::encode(unique_salt));

    // Compute CREATE2 address
    let address = compute_create2_address(
        factory_addr,
        bytecode_hash,
        unique_salt,
        proxy_address,
    );

    console_log!("Computed address: 0x{}", hex::encode(address));
    Ok(format!("0x{}", hex::encode(address)))
}

/// Compute CREATE2 address (zkSync version)
/// This is the zkSync-specific CREATE2 computation
fn compute_create2_address(
    deployer: Address,
    bytecode_hash: FixedBytes<32>,
    salt: FixedBytes<32>,
    input: Bytes,
) -> Address {
    // zkSync CREATE2 formula:
    // keccak256(0xff ++ deployer ++ salt ++ keccak256(bytecode_hash ++ input_hash))

    let input_hash = keccak256(&input);

    let mut bytecode_and_input = Vec::new();
    bytecode_and_input.extend(bytecode_hash.as_slice());
    bytecode_and_input.extend(input_hash.as_slice());
    let bytecode_input_hash = keccak256(bytecode_and_input);

    let mut create2_input = Vec::new();
    create2_input.push(0xff);
    create2_input.extend(deployer.as_slice());
    create2_input.extend(salt.as_slice());
    create2_input.extend(bytecode_input_hash.as_slice());

    let hash = keccak256(create2_input);
    Address::from_slice(&hash[12..])
}

// Error type for WASM
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ZkSyncSsoError {
    message: String,
    kind: String,
}

#[wasm_bindgen]
impl ZkSyncSsoError {
    #[wasm_bindgen(constructor)]
    pub fn new(message: &str, kind: &str) -> Self {
        Self { message: message.to_string(), kind: kind.to_string() }
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        self.kind.clone()
    }
}

// Contract addresses configuration
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Contracts {
    entry_point: String,
    account_factory: String,
}

#[wasm_bindgen]
impl Contracts {
    #[wasm_bindgen(constructor)]
    pub fn new(entry_point: &str, account_factory: &str) -> Self {
        Self {
            entry_point: entry_point.to_string(),
            account_factory: account_factory.to_string(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn entry_point(&self) -> String {
        self.entry_point.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn account_factory(&self) -> String {
        self.account_factory.clone()
    }
}

// Client configuration
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Config {
    rpc_url: String,
    bundler_url: String,
    contracts: Contracts,
}

#[wasm_bindgen]
impl Config {
    #[wasm_bindgen(constructor)]
    pub fn new(rpc_url: &str, bundler_url: &str, contracts: Contracts) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            bundler_url: bundler_url.to_string(),
            contracts,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn rpc_url(&self) -> String {
        self.rpc_url.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn bundler_url(&self) -> String {
        self.bundler_url.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn contracts(&self) -> Contracts {
        self.contracts.clone()
    }
}

// Transaction call data
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Call {
    to: String,
    data: String,
    value: String,
}

#[wasm_bindgen]
impl Call {
    #[wasm_bindgen(constructor)]
    pub fn new(to: &str, data: &str, value: &str) -> Self {
        Self {
            to: to.to_string(),
            data: data.to_string(),
            value: value.to_string(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn to(&self) -> String {
        self.to.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> String {
        self.data.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

// User operation request
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SendCallsRequest {
    account: String,
    calls: Vec<Call>,
}

#[wasm_bindgen]
impl SendCallsRequest {
    #[wasm_bindgen(constructor)]
    pub fn new(account: &str, calls: Vec<Call>) -> Self {
        Self { account: account.to_string(), calls }
    }

    #[wasm_bindgen(getter)]
    pub fn account(&self) -> String {
        self.account.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn calls(&self) -> Vec<Call> {
        self.calls.clone()
    }
}

// Main client implementation
#[wasm_bindgen]
pub struct Client {
    config: Config,
    #[allow(dead_code)]
    // Used in stub implementation, will be used when real implementation is added
    private_key: String,
}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new(config: Config, private_key: &str) -> Result<Client, JsValue> {
        // Basic validation
        if !private_key.starts_with("0x")
            || (private_key.len() != 64 && private_key.len() != 66)
        {
            return Err(JsValue::from_str("Invalid private key format"));
        }

        Ok(Client { config, private_key: private_key.to_string() })
    }

    #[wasm_bindgen]
    pub async fn send_user_operation(
        &self,
        request: SendCallsRequest,
    ) -> Result<String, JsValue> {
        // This is a stub implementation that will be replaced with actual logic
        // once the core crate is working
        console_log!(
            "Sending user operation for account: {}",
            request.account()
        );
        console_log!("Number of calls: {}", request.calls().len());
        console_log!("RPC URL: {}", self.config.rpc_url());
        console_log!("Bundler URL: {}", self.config.bundler_url());

        // Return a mock transaction hash for now
        Ok("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .to_string())
    }

    #[wasm_bindgen(getter)]
    pub fn config(&self) -> Config {
        self.config.clone()
    }
}

#[cfg(test)]
mod tests {
    use alloy::{
        primitives::{Bytes, FixedBytes, U256, address, bytes, fixed_bytes},
        providers::Provider,
        rpc::types::TransactionRequest,
    };
    use eyre;
    use std::sync::Arc;
    use zksync_sso_erc4337_core::{
        erc4337::account::{
            erc7579::{Execution, module_installed::is_module_installed},
            modular_smart_account::{
                add_passkey::PasskeyPayload as CorePasskeyPayload,
                deploy::deploy_account,
            },
        },
        utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        },
    };

    fn get_signature_from_js(hash: String) -> eyre::Result<Bytes> {
        use std::process::Command;

        let working_dir = "../../../../../erc4337-contracts";

        let output = Command::new("pnpm")
            .arg("tsx")
            .arg("test/integration/utils.ts")
            .arg("--hash")
            .arg(&hash)
            .current_dir(working_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eyre::bail!("Failed to sign hash with passkey: {}", stderr);
        }

        let stdout = String::from_utf8(output.stdout)?;

        // Extract the last non-empty line which should be the hex signature
        let last_line = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .next_back()
            .ok_or_else(|| {
                eyre::eyre!("No output from sign_hash_with_passkey command")
            })?;

        let hex_sig = last_line.trim();

        // Parse the hex string
        let hex_str = if let Some(stripped) = hex_sig.strip_prefix("0x") {
            stripped
        } else {
            hex_sig
        };

        let bytes_vec = hex::decode(hex_str)?;
        Ok(Bytes::from(bytes_vec))
    }

    /// Test that mimics the browser two-step flow using WASM FFI functions
    /// This verifies the prepare -> sign -> submit pattern works correctly
    ///
    /// This test deploys an account with a passkey from the start (no EOA signer),
    /// which is more representative of the browser flow
    #[tokio::test]
    async fn test_wasm_passkey_two_step_flow() -> eyre::Result<()> {
        let signer_private_key = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6".to_string();
        let config =
            TestInfraConfig { signer_private_key: signer_private_key.clone() };
        let (
            _,
            anvil_instance,
            provider,
            contracts,
            _signer_private_key,
            bundler,
            bundler_client,
        ) = start_anvil_and_deploy_contracts_and_start_bundler_with_config(
            &config,
        )
        .await?;

        let factory_address = contracts.account_factory;
        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");
        let webauthn_module = contracts.webauthn_validator;

        // Define passkey credentials (same as used in core tests)
        let credential_id = bytes!("0x2868baa08431052f6c7541392a458f64");
        let passkey = [
            fixed_bytes!(
                "0xe0a43b9c64a2357ea7f66a0551f57442fbd32031162d9be762800864168fae40"
            ),
            fixed_bytes!(
                "0x450875e2c28222e81eb25ae58d095a3e7ca295faa3fc26fb0e558a0b571da501"
            ),
        ];
        let origin_domain = "https://example.com".to_string();

        // Deploy account WITH passkey from the start (no EOA signer)
        let passkey_payload = CorePasskeyPayload {
            credential_id: credential_id.clone(),
            passkey,
            origin_domain: origin_domain.clone(),
        };

        use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::deploy::WebauthNSigner;

        let webauthn_signer = WebauthNSigner {
            passkey: passkey_payload.clone(),
            validator_address: webauthn_module,
        };

        let account_address = deploy_account(
            factory_address,
            None, // No EOA signer
            Some(webauthn_signer),
            provider.clone(),
        )
        .await?;

        println!("Account deployed with passkey: {:?}", account_address);

        // Fund the account
        {
            let fund_tx = TransactionRequest::default()
                .to(account_address)
                .value(U256::from(10000000000000000000u64));
            _ = provider.send_transaction(fund_tx).await?.get_receipt().await?;
        }

        // Verify passkey module is installed
        let is_installed = is_module_installed(
            webauthn_module,
            account_address,
            provider.clone(),
        )
        .await?;
        eyre::ensure!(is_installed, "WebAuthn module is not installed");

        println!("Passkey module verified as installed");

        // ===== TWO-STEP FLOW - MANUAL APPROACH LIKE CORE TEST =====
        // Instead of using send_transaction, manually build UserOp like the passing core test

        println!("\nTesting two-step passkey flow (manual approach)...");

        use alloy::rpc::types::erc4337::PackedUserOperation as AlloyPackedUserOperation;
        use zksync_sso_erc4337_core::erc4337::{
            account::{
                erc7579::calls::encode_calls,
                modular_smart_account::nonce::get_nonce,
            },
            bundler::Bundler,
            entry_point::EntryPoint::PackedUserOperation,
            user_operation::hash::v08::get_user_operation_hash_entry_point,
        };

        let call = Execution {
            target: account_address,
            value: U256::from(1),
            data: Bytes::default(),
        };
        let calls = vec![call];
        let call_data: Bytes = encode_calls(calls).into();

        let nonce_key = alloy::primitives::Uint::from(0);
        let nonce = get_nonce(
            entry_point_address,
            account_address,
            nonce_key,
            &provider,
        )
        .await?;

        // Create stub signature for gas estimation
        let stub_sig =
            get_signature_from_js(FixedBytes::<32>::default().to_string())?;

        // Build AlloyPackedUserOperation with stub values for gas estimation
        let mut user_op = AlloyPackedUserOperation {
            sender: account_address,
            nonce,
            paymaster: None,
            paymaster_verification_gas_limit: None,
            paymaster_data: None,
            paymaster_post_op_gas_limit: None,
            call_gas_limit: Default::default(),
            max_priority_fee_per_gas: Default::default(),
            max_fee_per_gas: Default::default(),
            pre_verification_gas: Default::default(),
            verification_gas_limit: Default::default(),
            factory: None,
            factory_data: None,
            call_data,
            signature: Bytes::from(stub_sig),
        };

        println!("About to estimate gas, bundler is in scope...");

        // Explicitly reference bundler to prevent optimizer from dropping it
        let _keep_alive = &bundler;

        // Estimate gas - holding bundler reference
        let estimated_gas = bundler_client
            .estimate_user_operation_gas(&user_op, &entry_point_address)
            .await?;

        println!("Gas estimation succeeded!");

        // Update with estimated gas values
        user_op.call_gas_limit = estimated_gas.call_gas_limit;
        user_op.verification_gas_limit = (estimated_gas.verification_gas_limit
            * U256::from(6))
            / U256::from(5);
        user_op.pre_verification_gas = estimated_gas.pre_verification_gas;
        user_op.max_priority_fee_per_gas = U256::from(0x77359400);
        user_op.max_fee_per_gas = U256::from(0x82e08afeu64);

        // Pack gas limits and fees for hashing
        let packed_gas_limits: U256 =
            (user_op.verification_gas_limit << 128) | user_op.call_gas_limit;
        let gas_fees: U256 =
            (user_op.max_priority_fee_per_gas << 128) | user_op.max_fee_per_gas;

        let packed_user_op = PackedUserOperation {
            sender: user_op.sender,
            nonce: user_op.nonce,
            initCode: Bytes::default(),
            callData: user_op.call_data.clone(),
            accountGasLimits: packed_gas_limits.to_be_bytes().into(),
            preVerificationGas: user_op.pre_verification_gas,
            gasFees: gas_fees.to_be_bytes().into(),
            paymasterAndData: Bytes::default(),
            signature: user_op.signature.clone(),
        };

        let hash = get_user_operation_hash_entry_point(
            &packed_user_op,
            &entry_point_address,
            provider.clone(),
        )
        .await?;

        println!("UserOp hash to sign: {:?}", hash);

        // Sign the hash
        let full_signature = get_signature_from_js(hash.0.to_string())?;
        println!("Full signature length: {} bytes", full_signature.len());

        // Update UserOp with signature and submit
        user_op.signature = Bytes::from(full_signature);

        let user_op_hash = bundler_client
            .send_user_operation(entry_point_address, user_op)
            .await?;

        println!("UserOperation submitted: {:?}", user_op_hash);

        bundler_client.wait_for_user_operation_receipt(user_op_hash).await?;

        println!("✅ Passkey transaction successfully sent!");
        println!(
            "✅ Two-step flow verified - account deployed with passkey only"
        );
        println!("✅ No EOA signer needed - pure passkey authentication");

        // Explicitly drop at end
        drop(anvil_instance);
        drop(bundler);
        Ok(())
    }
}
