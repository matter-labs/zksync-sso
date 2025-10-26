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

/// Prepare a UserOperation for passkey signing
/// Returns the hash that needs to be signed by the passkey
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
        use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::{
            nonce::get_nonce,
            signature::stub_signature_passkey,
        };
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

        console_log!("  Using stub signature for gas estimation: {} bytes", stub_sig.len());

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
            Bundler,
            pimlico::client::{BundlerClient, BundlerConfig},
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

        console_log!("  Estimated gas: call={}, verification={}, preVerification={}", 
            estimated_gas.call_gas_limit,
            estimated_gas.verification_gas_limit,
            estimated_gas.pre_verification_gas
        );

        // Update with estimated gas values (with 20% buffer on verification gas like the working test)
        user_op.call_gas_limit = estimated_gas.call_gas_limit;
        user_op.verification_gas_limit =
            (estimated_gas.verification_gas_limit * U256::from(6)) / U256::from(5);
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

        // Update UserOperation with full signature (validator address + passkey signature)
        user_op.signature = Bytes::from(full_signature);

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
