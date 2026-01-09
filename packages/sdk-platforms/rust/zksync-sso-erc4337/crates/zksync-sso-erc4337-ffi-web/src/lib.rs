// WASM transport is implemented but not yet fully integrated with Alloy's Provider trait
// For now, we expose offline computation functions
mod account;
mod wasm_transport;

use alloy::{
    network::EthereumWallet,
    primitives::{Address, B256, Bytes, FixedBytes, U256, Uint, keccak256},
    providers::ProviderBuilder,
    rpc::types::erc4337::PackedUserOperation as AlloyPackedUserOperation,
    signers::local::PrivateKeySigner,
    sol,
    sol_types::SolValue,
};
use alloy_rpc_client::RpcClient;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use wasm_transport::WasmHttpTransport;
use zksync_sso_erc4337_core::{
    chain::{Chain, id::ChainId},
    config::contracts::Contracts as CoreContracts,
    erc4337::{
        account::{
            erc7579::calls::encoded_call_data as encoded_call_data_core,
            modular_smart_account::{
                deploy::{
                    DeployAccountParams as DeployAccountParamsCore,
                    EOASigners as CoreEOASigners,
                    WebAuthNSigner as CoreWebauthNSigner,
                    deploy_account as deploy_account_core,
                    deploy_accout_call_data,
                    initialize_account_call_data as initialize_account_call_data_core,
                },
                passkey::{
                    add::{
                        AddPasskeyParams as AddPasskeyParamsCore,
                        PasskeyPayload as CorePasskeyPayload,
                        add_passkey as add_passkey_core,
                        add_passkey_call_data as add_passkey_call_data_core,
                    },
                    list::get_account_list_call_data as get_account_list_call_data_core,
                },
                send::{
                    SendUserOpParams, eoa::EOASendParams,
                    send_user_op as send_user_op_core,
                },
                session::{
                    create::create_session_call_data as create_session_call_data_core,
                    session_lib::session_spec::SessionSpec,
                    state::session_state_call_data as session_state_call_data_core,
                },
                signers::{
                    eoa::{
                        eoa_sign as eoa_sign_core,
                        eoa_signature as eoa_signature_core,
                        stub_signature_eoa as stub_signature_eoa_core,
                    },
                    passkey::stub_signature_passkey as stub_signature_passkey_core,
                },
            },
        },
        bundler::{
            Bundler, config::BundlerConfig as BundlerConfigCore,
            pimlico::client::BundlerClient as BundlerClientCore,
        },
        entry_point::{
            PackedUserOperation,
            nonce::{
                GetNonceWithKeyParams, get_nonce_call_data,
                get_nonce_with_key as get_nonce_with_key_core,
            },
            user_op_hash::get_user_op_hash_call_data,
            version::EntryPointVersion,
        },
        paymaster::params::PaymasterParams as PaymasterParamsCore,
        signer::create_eoa_signer,
        user_operation::hash::user_operation_hash::get_user_operation_hash_entry_point as get_user_operation_hash_entry_point_core,
    },
};

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
                Err(JsValue::from_str(&format!("RPC request failed: {}", e)))
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

// Paymaster parameters for sponsoring transactions
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct PaymasterParams {
    address: String,
    data: String,
    verification_gas_limit: Option<String>,
    post_op_gas_limit: Option<String>,
}

#[wasm_bindgen]
impl PaymasterParams {
    #[wasm_bindgen(constructor)]
    pub fn new(
        address: String,
        data: Option<String>,
        verification_gas_limit: Option<String>,
        post_op_gas_limit: Option<String>,
    ) -> Self {
        Self {
            address,
            data: data.unwrap_or_default(),
            verification_gas_limit,
            post_op_gas_limit,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn address(&self) -> String {
        self.address.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> String {
        self.data.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn verification_gas_limit(&self) -> Option<String> {
        self.verification_gas_limit.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn post_op_gas_limit(&self) -> Option<String> {
        self.post_op_gas_limit.clone()
    }
}

// Normalize/convert PaymasterParams into core params, or None if effectively unset
fn normalize_paymaster_params(
    pm: Option<PaymasterParams>,
) -> Result<Option<PaymasterParamsCore>, String> {
    match pm {
        None => Ok(None),
        Some(pm) => {
            // Treat empty address or zero address with no gas hints as "not provided"
            let addr_parsed = pm.address().parse::<Address>().map_err(|e| {
                format!("Invalid paymaster address '{}': {}", pm.address(), e)
            })?;

            let has_gas_hints = pm
                .verification_gas_limit()
                .as_ref()
                .map(|s| !s.is_empty())
                .unwrap_or(false)
                || pm
                    .post_op_gas_limit()
                    .as_ref()
                    .map(|s| !s.is_empty())
                    .unwrap_or(false);

            // Empty/zero address and no explicit gas hints => ignore paymaster
            if addr_parsed == Address::ZERO && !has_gas_hints {
                return Ok(None);
            }

            let data_bytes = if pm.data().is_empty() {
                Bytes::default()
            } else {
                match hex::decode(pm.data().trim_start_matches("0x")) {
                    Ok(bytes) => Bytes::from(bytes),
                    Err(_) => Bytes::default(),
                }
            };

            let verification_gas = pm
                .verification_gas_limit()
                .and_then(|s| s.parse::<U256>().ok());

            let post_op_gas =
                pm.post_op_gas_limit().and_then(|s| s.parse::<U256>().ok());

            Ok(Some(PaymasterParamsCore {
                address: addr_parsed,
                data: data_bytes,
                verification_gas_limit: verification_gas,
                post_op_gas_limit: post_op_gas,
            }))
        }
    }
}

/// Prepared UserOperation data that can be passed to submit after signing
/// This eliminates the need for global state storage
#[wasm_bindgen]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PreparedUserOperation {
    sender: String,
    nonce: String,
    call_data: String,
    call_gas_limit: String,
    verification_gas_limit: String,
    pre_verification_gas: String,
    max_priority_fee_per_gas: String,
    max_fee_per_gas: String,
    validator_address: String,
    paymaster: Option<String>,
    paymaster_data: Option<String>,
    paymaster_verification_gas_limit: Option<String>,
    paymaster_post_op_gas_limit: Option<String>,
}

#[wasm_bindgen]
impl PreparedUserOperation {
    /// Create from JSON string (for JavaScript integration)
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<PreparedUserOperation, JsValue> {
        serde_json::from_str(json).map_err(|e| {
            JsValue::from_str(&format!("Failed to parse JSON: {}", e))
        })
    }

    /// Convert to JSON string
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(self).map_err(|e| {
            JsValue::from_str(&format!("Failed to serialize JSON: {}", e))
        })
    }

    // Getters for JavaScript
    #[wasm_bindgen(getter)]
    pub fn sender(&self) -> String {
        self.sender.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn nonce(&self) -> String {
        self.nonce.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn call_data(&self) -> String {
        self.call_data.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn call_gas_limit(&self) -> String {
        self.call_gas_limit.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn verification_gas_limit(&self) -> String {
        self.verification_gas_limit.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn pre_verification_gas(&self) -> String {
        self.pre_verification_gas.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn max_priority_fee_per_gas(&self) -> String {
        self.max_priority_fee_per_gas.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn max_fee_per_gas(&self) -> String {
        self.max_fee_per_gas.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn validator_address(&self) -> String {
        self.validator_address.clone()
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
                    return Err(JsValue::from_str(&format!(
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
                return Err(JsValue::from_str(&format!(
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
                            return Err(JsValue::from_str(&format!(
                                "Invalid EOA signer address '{}': {}",
                                addr_str, e
                            )));
                        }
                    }
                }

                let validator_addr = match validator.parse::<Address>() {
                    Ok(addr) => addr,
                    Err(e) => {
                        return Err(JsValue::from_str(&format!(
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
                return Err(JsValue::from_str(
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
                    return Err(JsValue::from_str(&format!(
                        "Invalid passkey X coordinate length: expected 32 bytes, got {}",
                        passkey.passkey_x.len()
                    )));
                }
                if passkey.passkey_y.len() != 32 {
                    return Err(JsValue::from_str(&format!(
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
                        return Err(JsValue::from_str(&format!(
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
                return Err(JsValue::from_str(
                    "Both passkey_payload and webauthn_validator_address must be provided together",
                ));
            }
        };

        console_log!("  Calling core deploy_account...");

        // Use the core crate's deploy_account function
        match deploy_account_core(DeployAccountParamsCore {
            factory_address: factory_addr,
            eoa_signers,
            webauthn_signer,
            session_validator: None,
            id: None,
            provider,
        })
        .await
        {
            Ok(address) => {
                let address_str = format!("0x{:x}", address);
                console_log!("  Deployed account address: {}", address_str);
                Ok(JsValue::from_str(&address_str))
            }
            Err(e) => {
                console_log!("  Error deploying account: {}", e);
                Err(JsValue::from_str(&format!(
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
                return Err(JsValue::from_str(&format!(
                    "Invalid account address: {}",
                    e
                )));
            }
        };

        let entry_point = match config.entry_point_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid entry point address: {}",
                    e
                )));
            }
        };

        let webauthn_validator =
            match webauthn_validator_address.parse::<Address>() {
                Ok(addr) => addr,
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Invalid WebAuthn validator address: {}",
                        e
                    )));
                }
            };

        let eoa_validator = match eoa_validator_address.parse::<Address>() {
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
        let transport = WasmHttpTransport::new(config.rpc_url);
        let client = RpcClient::new(transport.clone(), false);
        let provider =
            ProviderBuilder::new().wallet(eoa_wallet).connect_client(client);

        // Create bundler client
        let bundler_client = {
            let config = BundlerConfigCore::new(config.bundler_url);
            zksync_sso_erc4337_core::erc4337::bundler::pimlico::client::BundlerClient::new(config)
        };

        // Create EOA signer
        let eoa_key_str = format!("0x{}", hex::encode(eoa_key.to_bytes()));
        let signer = match create_eoa_signer(eoa_key_str, eoa_validator) {
            Ok(signer) => signer,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Failed to create EOA signer: {}",
                    e
                )));
            }
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
        match add_passkey_core(AddPasskeyParamsCore {
            account_address: account,
            passkey: core_passkey,
            webauthn_validator,
            entry_point_address: entry_point,
            paymaster: None,
            provider,
            bundler_client,
            signer,
        })
        .await
        {
            Ok(_) => {
                console_log!("  Passkey added successfully");
                Ok(JsValue::from_str("Passkey registered successfully"))
            }
            Err(e) => {
                console_log!("  Error adding passkey: {}", e);
                Err(JsValue::from_str(&format!("Failed to add passkey: {}", e)))
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
#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn send_transaction_eoa(
    config: SendTransactionConfig,
    eoa_validator_address: String,
    eoa_private_key: String,
    account_address: String,
    to_address: String,
    value: String,
    data: Option<String>,
    paymaster: Option<PaymasterParams>,
) -> js_sys::Promise {
    future_to_promise(async move {
        console_log!("Starting EOA transaction...");
        console_log!("  Account: {}", account_address);
        console_log!("  To: {}", to_address);
        console_log!("  Value: {}", value);
        console_log!("  Bundler: {}", config.bundler_url);
        if let Some(ref pm) = paymaster {
            console_log!("  Paymaster: {}", pm.address());
        }

        // Parse addresses
        let account = match account_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid account address: {}",
                    e
                )));
            }
        };

        let entry_point = match config.entry_point_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid entry point address: {}",
                    e
                )));
            }
        };

        let eoa_validator = match eoa_validator_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid EOA validator address: {}",
                    e
                )));
            }
        };

        let to = match to_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid to address: {}",
                    e
                )));
            }
        };

        // Parse value
        let value_u256 = match value.parse::<U256>() {
            Ok(v) => v,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid value: {}",
                    e
                )));
            }
        };

        // Parse data
        let data_bytes = match data {
            Some(d) => {
                let hex_str = d.trim_start_matches("0x");
                match hex::decode(hex_str) {
                    Ok(bytes) => Bytes::from(bytes),
                    Err(e) => {
                        return Err(JsValue::from_str(&format!(
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
                return Err(JsValue::from_str(&format!(
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
            let config = BundlerConfigCore::new(config.bundler_url);
            BundlerClientCore::new(config)
        };

        console_log!("  Created bundler client");

        // Encode the execution call
        let encoded_calls: Bytes =
            encoded_call_data_core(to, Some(data_bytes), Some(value_u256));

        // Convert optional paymaster params, ignoring effectively empty values
        let paymaster_params = normalize_paymaster_params(paymaster)
            .map_err(|e| JsValue::from_str(&e))?;

        console_log!(
            "  Encoded call data, calling core send_transaction_eoa..."
        );

        match zksync_sso_erc4337_core::erc4337::account::modular_smart_account::send::eoa::eoa_send_transaction(EOASendParams {
            account,
            entry_point,
            call_data: encoded_calls,
            nonce_key: None,
            paymaster: paymaster_params,
            bundler_client,
            provider,
            eoa_validator,
            private_key_hex: eoa_private_key,
        })
        .await
        {
            Ok(_) => {
                console_log!("  Transaction sent successfully!");
                Ok(JsValue::from_str("Transaction sent successfully"))
            }
            Err(e) => {
                console_log!("  Error sending transaction: {}", e);
                Err(JsValue::from_str(&format!("Failed to send transaction: {}", e)))
            }
        }
    })
}

/// Send a UserOperation with paymaster support
///
/// This is the core function that handles bundler communication with optional paymaster sponsorship.
/// It constructs the complete UserOperation, estimates gas, and submits to the bundler.
///
/// # Parameters
/// * `config` - SendTransactionConfig with RPC, bundler, and entry point
/// * `account_address` - The smart account address
/// * `call_data` - The encoded call data
/// * `private_key_hex` - EOA private key for signing (0x-prefixed)
/// * `validator_address` - The validator module address
/// * `paymaster` - Optional PaymasterParams for sponsoring the transaction
///
/// # Returns
/// Promise that resolves with the UserOperation receipt
#[wasm_bindgen]
pub fn send_user_operation(
    config: SendTransactionConfig,
    account_address: String,
    call_data: String,
    private_key_hex: String,
    validator_address: String,
    paymaster: Option<PaymasterParams>,
) -> js_sys::Promise {
    future_to_promise(async move {
        console_log!("Starting user operation...");
        console_log!("  Account: {}", account_address);
        console_log!("  Validator: {}", validator_address);
        if let Some(ref pm) = paymaster {
            console_log!("  Paymaster: {}", pm.address());
        }

        // Parse addresses
        let account: Address = match account_address.parse() {
            Ok(addr) => addr,
            Err(_) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid account address: {}",
                    account_address
                )));
            }
        };

        let entry_point: Address = match config.entry_point_address.parse() {
            Ok(addr) => addr,
            Err(_) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid entry point address: {}",
                    config.entry_point_address
                )));
            }
        };

        let validator: Address = match validator_address.parse() {
            Ok(addr) => addr,
            Err(_) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid validator address: {}",
                    validator_address
                )));
            }
        };

        // Parse call data
        let call_data_bytes = match call_data
            .trim_start_matches("0x")
            .is_empty()
        {
            true => Bytes::default(),
            false => match hex::decode(call_data.trim_start_matches("0x")) {
                Ok(bytes) => Bytes::from(bytes),
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Invalid call data hex: {}",
                        e
                    )));
                }
            },
        };

        // Parse private key
        let eoa_key = match private_key_hex
            .trim_start_matches("0x")
            .parse::<PrivateKeySigner>()
        {
            Ok(signer) => signer,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid private key: {}",
                    e
                )));
            }
        };

        let eoa_wallet = EthereumWallet::from(eoa_key);

        // Create transport and provider
        let transport = WasmHttpTransport::new(config.rpc_url.clone());
        let client = RpcClient::new(transport, false);
        let provider =
            ProviderBuilder::new().wallet(eoa_wallet).connect_client(client);

        // Create bundler client
        let bundler_client = {
            let bundler_config =
                BundlerConfigCore::new(config.bundler_url.clone());
            BundlerClientCore::new(bundler_config)
        };

        // Create signer
        let signer = match create_eoa_signer(private_key_hex, validator) {
            Ok(s) => s,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Failed to create signer: {}",
                    e
                )));
            }
        };

        // Convert PaymasterParams if provided, ignoring effectively empty values
        let paymaster_params = normalize_paymaster_params(paymaster)
            .map_err(|e| JsValue::from_str(&e))?;

        console_log!("  Calling send_user_op with bundler...");

        // Call the core send_user_op function
        match send_user_op_core(SendUserOpParams {
            account,
            entry_point,
            factory_payload: None,
            call_data: call_data_bytes,
            nonce_key: None,
            paymaster: paymaster_params,
            bundler_client,
            provider,
            signer,
        })
        .await
        {
            Ok(receipt) => {
                console_log!("  UserOperation sent successfully!");
                console_log!("  Receipt: {:?}", receipt);
                Ok(JsValue::from_str(
                    &serde_json::to_string(&receipt)
                        .unwrap_or_else(|_| "Success".to_string()),
                ))
            }
            Err(e) => {
                console_log!("  Error sending user operation: {}", e);
                Err(JsValue::from_str(&format!(
                    "Failed to send user operation: {}",
                    e
                )))
            }
        }
    })
}

/// Prepare a UserOperation for passkey signing with fixed gas limits
/// Returns the hash that needs to be signed by the passkey and the prepared UserOperation data
///
/// This function creates a stub signature internally, so the caller doesn't need to provide one.
/// The prepared UserOperation data should be stored by the caller and passed to submit_passkey_user_operation
/// after the passkey signature is obtained (stateless design - no global state).
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
/// JSON string with format: `{"hash": "0x...", "userOp": {...}}`
/// - hash: The hash that should be signed by the passkey
/// - userOp: The prepared UserOperation data (serialize this and pass to submit_passkey_user_operation)
#[wasm_bindgen]
pub fn prepare_passkey_user_operation(
    config: SendTransactionConfig,
    webauthn_validator_address: String,
    account_address: String,
    to_address: String,
    value: String,
    data: Option<String>,
    paymaster: Option<PaymasterParams>,
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
                return Err(JsValue::from_str(&format!(
                    "Invalid account address: {}",
                    e
                )));
            }
        };

        let entry_point = match config.entry_point_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid entry point address: {}",
                    e
                )));
            }
        };

        let to = match to_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid to address: {}",
                    e
                )));
            }
        };

        // Parse value
        let value_u256 = match value.parse::<U256>() {
            Ok(v) => v,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid value: {}",
                    e
                )));
            }
        };

        // Parse data
        let data_bytes = match data {
            Some(d) => {
                let hex_str = d.trim_start_matches("0x");
                match hex::decode(hex_str) {
                    Ok(bytes) => Bytes::from(bytes),
                    Err(e) => {
                        return Err(JsValue::from_str(&format!(
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

        // Convert optional paymaster params, ignoring effectively empty values
        let paymaster_params = normalize_paymaster_params(paymaster)
            .map_err(|e| JsValue::from_str(&e))?;

        console_log!("  Created provider and transport");

        // Encode the execution call
        let encoded_calls: Bytes =
            encoded_call_data_core(to, Some(data_bytes), Some(value_u256));

        console_log!("  Encoded call data");

        // Build UserOperation with fixed high gas values (no bundler estimation)

        let nonce_key = Uint::from(0);
        let nonce = match get_nonce_with_key_core(GetNonceWithKeyParams {
            sender: account,
            entry_point,
            key: nonce_key,
            provider: provider.clone(),
        })
        .await
        {
            Ok(n) => n,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Failed to get nonce: {}",
                    e
                )));
            }
        };

        // Parse validator address to create stub signature
        let validator = match webauthn_validator_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid validator address: {}",
                    e
                )));
            }
        };

        // Create stub signature internally (validator address + minimal ABI-encoded data)
        // This matches the format expected by WebAuthnValidator: (bytes authenticatorData, string clientDataJSON, bytes32[2] rs, bytes credentialId)
        let stub_sig = match stub_signature_passkey_core(validator) {
            Ok(sig) => sig,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Failed to create stub signature: {}",
                    e
                )));
            }
        };

        console_log!("  Created stub signature: {} bytes", stub_sig.len());

        // Use fixed high gas values (matching the Rust test approach when bundler is unavailable)
        // Increase verification gas for paymaster transactions since they require additional validation
        let call_gas_limit = U256::from(3_000_000u64); // Increased from 2M
        let verification_gas_limit = if paymaster_params.is_some() {
            U256::from(5_000_000u64) // Increased from 3M for paymaster + passkey validation
        } else {
            U256::from(3_000_000u64) // Increased from 2M
        };
        let pre_verification_gas = U256::from(1_500_000u64); // Increased from 1M
        let max_priority_fee_per_gas = U256::from(0x77359400u64);
        let max_fee_per_gas = U256::from(0x82e08afeu64);

        // Paymaster fields (optional)
        let (
            paymaster_addr_opt,
            paymaster_data_opt,
            paymaster_ver_gas_opt,
            paymaster_post_gas_opt,
        ) = match paymaster_params {
            Some(ref pm) => {
                // Auto-set paymaster_post_op_gas_limit if not provided (default: 1M gas)
                let post_op_gas = pm
                    .post_op_gas_limit
                    .or_else(|| Some(U256::from(1_000_000u64)));
                (
                    Some(pm.address),
                    Some(pm.data.clone()),
                    pm.verification_gas_limit,
                    post_op_gas,
                )
            }
            None => (None, None, None, None),
        };

        console_log!(
            "  Using fixed gas limits: call={}, verification={}, preVerification={}",
            call_gas_limit,
            verification_gas_limit,
            pre_verification_gas
        );
        if paymaster_addr_opt.is_some() {
            console_log!(
                "  Paymaster gas limits: verification={}, postOp={}",
                paymaster_ver_gas_opt.unwrap_or(U256::ZERO),
                paymaster_post_gas_opt.unwrap_or(U256::ZERO)
            );
        }

        // Create AlloyPackedUserOperation
        let user_op = AlloyPackedUserOperation {
            sender: account,
            nonce,
            call_data: encoded_calls.clone(),
            signature: stub_sig.clone(),
            paymaster: paymaster_addr_opt,
            paymaster_verification_gas_limit: paymaster_ver_gas_opt,
            paymaster_data: paymaster_data_opt.clone(),
            paymaster_post_op_gas_limit: paymaster_post_gas_opt,
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
        // Use the same build_paymaster_and_data function as the core send module
        let paymaster_and_data = zksync_sso_erc4337_core::erc4337::paymaster::params::build_paymaster_and_data(
            paymaster_addr_opt,
            paymaster_ver_gas_opt,
            paymaster_post_gas_opt,
            paymaster_data_opt.as_ref(),
        );

        let packed_user_op = PackedUserOperation {
            sender: user_op.sender,
            nonce: user_op.nonce,
            initCode: Bytes::default(),
            callData: user_op.call_data.clone(),
            accountGasLimits: packed_gas_limits.to_be_bytes().into(),
            preVerificationGas: user_op.pre_verification_gas,
            gasFees: gas_fees.to_be_bytes().into(),
            paymasterAndData: paymaster_and_data.clone(),
            signature: user_op.signature.clone(),
        };

        // Get the hash that needs to be signed
        let hash = match get_user_operation_hash_entry_point_core(
            &packed_user_op,
            &entry_point,
            provider.clone(),
        )
        .await
        {
            Ok(h) => h,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Failed to get UserOp hash: {}",
                    e
                )));
            }
        };

        console_log!("  UserOperation hash: {:?}", hash);

        // Convert hash to B256 for hex format
        let hash_b256: B256 = hash.into();
        let hash_hex = format!("{:#x}", hash_b256);

        // Create PreparedUserOperation struct to return to JavaScript (stateless design)
        let prepared = PreparedUserOperation {
            sender: format!("{:#x}", account),
            nonce: user_op.nonce.to_string(),
            call_data: format!("0x{}", hex::encode(&user_op.call_data)),
            call_gas_limit: user_op.call_gas_limit.to_string(),
            verification_gas_limit: user_op.verification_gas_limit.to_string(),
            pre_verification_gas: user_op.pre_verification_gas.to_string(),
            max_priority_fee_per_gas: user_op
                .max_priority_fee_per_gas
                .to_string(),
            max_fee_per_gas: user_op.max_fee_per_gas.to_string(),
            validator_address: format!("{:#x}", validator),
            paymaster: paymaster_addr_opt.map(|a| format!("{:#x}", a)),
            paymaster_data: paymaster_data_opt
                .as_ref()
                .map(|d| format!("0x{}", hex::encode(d))),
            paymaster_verification_gas_limit: paymaster_ver_gas_opt
                .map(|g| g.to_string()),
            paymaster_post_op_gas_limit: paymaster_post_gas_opt
                .map(|g| g.to_string()),
        };

        // Serialize to JSON
        let prepared_json = match serde_json::to_string(&prepared) {
            Ok(json) => json,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Failed to serialize UserOperation: {}",
                    e
                )));
            }
        };

        // Return JSON with hash and prepared UserOperation data
        let result =
            format!(r#"{{"hash":"{}","userOp":{}}}"#, hash_hex, prepared_json);

        console_log!(
            "  Prepared UserOperation with fixed gas, waiting for passkey signature"
        );
        Ok(JsValue::from_str(&result))
    })
}

/// Submit a passkey-signed UserOperation
///
/// # Parameters
/// * `config` - SendTransactionConfig with bundler URL
/// * `prepared_user_op_json` - JSON string containing the prepared UserOperation data
/// * `passkey_signature` - The signature from WebAuthn (hex string with 0x prefix)
///
/// # Returns
/// Promise that resolves when the UserOperation is confirmed
#[wasm_bindgen]
pub fn submit_passkey_user_operation(
    config: SendTransactionConfig,
    prepared_user_op_json: String,
    passkey_signature: String,
) -> js_sys::Promise {
    future_to_promise(async move {
        console_log!("Submitting passkey-signed UserOperation...");

        // Parse the PreparedUserOperation from JSON
        let prepared: PreparedUserOperation =
            serde_json::from_str(&prepared_user_op_json).map_err(|e| {
                JsValue::from_str(&format!(
                    "Failed to parse prepared_user_op_json: {}",
                    e
                ))
            })?;
        console_log!("  Parsed prepared UserOperation");
        console_log!("    sender: {}", prepared.sender);
        console_log!("    nonce: {}", prepared.nonce);

        // Parse addresses
        let sender = match prepared.sender.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid sender address: {}",
                    e
                )));
            }
        };

        let validator_address =
            match prepared.validator_address.parse::<Address>() {
                Ok(addr) => addr,
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Invalid validator address: {}",
                        e
                    )));
                }
            };

        // Parse paymaster fields if present
        let paymaster_address = match prepared.paymaster {
            Some(ref addr) => match addr.parse::<Address>() {
                Ok(a) => Some(a),
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Invalid paymaster address: {}",
                        e
                    )));
                }
            },
            None => None,
        };

        let paymaster_data = match prepared.paymaster_data {
            Some(ref data_hex) => {
                let hex_str = data_hex.trim_start_matches("0x");
                match hex::decode(hex_str) {
                    Ok(bytes) => Some(Bytes::from(bytes)),
                    Err(e) => {
                        return Err(JsValue::from_str(&format!(
                            "Invalid paymaster_data hex: {}",
                            e
                        )));
                    }
                }
            }
            None => None,
        };

        let paymaster_verification_gas_limit = prepared
            .paymaster_verification_gas_limit
            .as_ref()
            .and_then(|v| v.parse::<U256>().ok());
        let paymaster_post_op_gas_limit = prepared
            .paymaster_post_op_gas_limit
            .as_ref()
            .and_then(|v| v.parse::<U256>().ok());

        // Parse numeric fields
        let nonce = match prepared.nonce.parse::<U256>() {
            Ok(n) => n,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid nonce: {}",
                    e
                )));
            }
        };

        let call_gas_limit = match prepared.call_gas_limit.parse::<U256>() {
            Ok(n) => n,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid call_gas_limit: {}",
                    e
                )));
            }
        };

        let verification_gas_limit =
            match prepared.verification_gas_limit.parse::<U256>() {
                Ok(n) => n,
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Invalid verification_gas_limit: {}",
                        e
                    )));
                }
            };

        let pre_verification_gas =
            match prepared.pre_verification_gas.parse::<U256>() {
                Ok(n) => n,
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Invalid pre_verification_gas: {}",
                        e
                    )));
                }
            };

        let max_priority_fee_per_gas =
            match prepared.max_priority_fee_per_gas.parse::<U256>() {
                Ok(n) => n,
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Invalid max_priority_fee_per_gas: {}",
                        e
                    )));
                }
            };

        let max_fee_per_gas = match prepared.max_fee_per_gas.parse::<U256>() {
            Ok(n) => n,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid max_fee_per_gas: {}",
                    e
                )));
            }
        };

        // Parse call data
        let call_data_hex = prepared.call_data.trim_start_matches("0x");
        let call_data = match hex::decode(call_data_hex) {
            Ok(bytes) => Bytes::from(bytes),
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid call_data hex: {}",
                    e
                )));
            }
        };

        // Parse the passkey signature
        let sig_hex = passkey_signature.trim_start_matches("0x");
        let signature = match hex::decode(sig_hex) {
            Ok(bytes) => Bytes::from(bytes),
            Err(e) => {
                return Err(JsValue::from_str(&format!(
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

        // Create UserOperation with the real signature
        let user_op = AlloyPackedUserOperation {
            sender,
            nonce,
            call_data,
            signature: Bytes::from(full_signature.clone()),
            paymaster: paymaster_address,
            paymaster_verification_gas_limit,
            paymaster_data: paymaster_data.clone(),
            paymaster_post_op_gas_limit,
            call_gas_limit,
            max_priority_fee_per_gas,
            max_fee_per_gas,
            pre_verification_gas,
            verification_gas_limit,
            factory: None,
            factory_data: None,
        };

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
            let config = BundlerConfigCore::new(config.bundler_url);
            BundlerClientCore::new(config)
        };

        // Parse entry point address for bundler call
        let entry_point = match config.entry_point_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid entry point address: {}",
                    e
                )));
            }
        };

        // Submit UserOperation

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
                        Err(JsValue::from_str(&format!(
                            "UserOperation submitted but failed to get receipt: {}",
                            e
                        )))
                    }
                }
            }
            Err(e) => {
                console_log!("  Error submitting UserOperation: {}", e);
                Err(JsValue::from_str(&format!(
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
    guardian_executor: &str,
) -> Result<String, JsValue> {
    match CoreContracts::from_string(
        entry_point.to_string(),
        account_factory.to_string(),
        webauthn_validator.to_string(),
        eoa_validator.to_string(),
        session_validator.to_string(),
        guardian_executor.to_string(),
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

// ===== NEW SDK HELPER FUNCTIONS =====
// These functions are used by the new viem-based SDK (packages/sdk/src/client-new)

/// Encode a call to EntryPoint.getNonce(sender, key)
/// Used for querying account nonce from the entry point
#[wasm_bindgen]
pub fn encode_get_nonce_call_data(
    sender: String,
    key: String,
) -> Result<String, JsValue> {
    // Parse sender address
    let sender_addr = sender.parse::<Address>().map_err(|e| {
        JsValue::from_str(&format!("Invalid sender address: {}", e))
    })?;

    // Parse key (should be a number string, default to 0)
    let key_u192 = key
        .parse::<alloy::primitives::Uint<192, 3>>()
        .unwrap_or(alloy::primitives::Uint::from(0));

    // Encode the call
    let encoded = get_nonce_call_data(sender_addr, key_u192);
    Ok(format!("0x{}", hex::encode(encoded)))
}

/// Decode the result of EntryPoint.getNonce() call
/// Returns the nonce as a string
#[wasm_bindgen]
pub fn decode_nonce_result(result: String) -> Result<String, JsValue> {
    // Remove 0x prefix if present
    let hex_str = result.trim_start_matches("0x");

    // Decode hex to bytes
    let bytes = hex::decode(hex_str)
        .map_err(|e| JsValue::from_str(&format!("Invalid hex: {}", e)))?;

    // Convert bytes to U256
    let nonce = U256::from_be_slice(&bytes);

    Ok(nonce.to_string())
}

/// Encode a call to WebAuthnValidator.getAccountList(string domain, bytes credentialId)
/// Used for querying which accounts are associated with a passkey credential
#[wasm_bindgen]
pub fn encode_get_account_list_call_data(
    domain: String,
    credential_id: String,
) -> Result<String, JsValue> {
    // Parse credential_id (can be 0x-prefixed or not)
    let cred_id_hex = credential_id.trim_start_matches("0x");
    let cred_id_bytes = hex::decode(cred_id_hex).map_err(|e| {
        JsValue::from_str(&format!("Invalid credential_id hex: {}", e))
    })?;

    let encoded = get_account_list_call_data_core(domain, cred_id_bytes);
    Ok(format!("0x{}", hex::encode(encoded)))
}

/// Decode the result of WebAuthnValidator.getAccountList() call
/// Returns JSON array of account addresses
#[wasm_bindgen]
pub fn decode_get_account_list_result(
    result: String,
) -> Result<String, JsValue> {
    use alloy::sol_types::SolType;

    // Remove 0x prefix if present
    let hex_str = result.trim_start_matches("0x");

    // Decode hex to bytes
    let bytes = hex::decode(hex_str)
        .map_err(|e| JsValue::from_str(&format!("Invalid hex: {}", e)))?;

    // Decode address[] (dynamic array of addresses)
    type AddressArray =
        alloy::sol_types::sol_data::Array<alloy::sol_types::sol_data::Address>;
    let addresses = <AddressArray as SolType>::abi_decode(&bytes)
        .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

    // Convert to hex strings with checksum
    let address_strings: Vec<String> = addresses
        .iter()
        .map(|addr| format!("{:?}", addr)) // Uses alloy's Debug impl which adds 0x and checksum
        .collect();

    // Return as JSON array
    serde_json::to_string(&address_strings)
        .map_err(|e| JsValue::from_str(&format!("JSON error: {}", e)))
}

/// Encode a single execute() call for the smart account
/// Used for encoding transaction calldata
#[wasm_bindgen]
pub fn encode_execute_call_data(
    target: String,
    value: String,
    data: String,
) -> Result<String, JsValue> {
    // Parse target address
    let target_addr = target.parse::<Address>().map_err(|e| {
        JsValue::from_str(&format!("Invalid target address: {}", e))
    })?;

    // Parse value
    let value_u256 = value
        .parse::<U256>()
        .map_err(|e| JsValue::from_str(&format!("Invalid value: {}", e)))?;

    // Parse data (hex string)
    let data_hex = data.trim_start_matches("0x");
    let data_bytes = if data_hex.is_empty() {
        Bytes::default()
    } else {
        let bytes_vec = hex::decode(data_hex).map_err(|e| {
            JsValue::from_str(&format!("Invalid data hex: {}", e))
        })?;
        Bytes::from(bytes_vec)
    };

    // Encode single call
    let encoded =
        encoded_call_data_core(target_addr, Some(data_bytes), Some(value_u256));

    Ok(format!("0x{}", hex::encode(encoded)))
}

/// Generate a stub signature for gas estimation with EOA validator
/// Returns a properly formatted signature for the validator
#[wasm_bindgen]
pub fn generate_eoa_stub_signature(
    validator_address: String,
) -> Result<String, JsValue> {
    // Parse validator address
    let validator_addr = validator_address.parse::<Address>().map_err(|e| {
        JsValue::from_str(&format!("Invalid validator address: {}", e))
    })?;

    // Generate stub signature
    let signature = stub_signature_eoa_core(validator_addr).map_err(|e| {
        JsValue::from_str(&format!("Failed to generate stub signature: {}", e))
    })?;

    Ok(format!("0x{}", hex::encode(&signature)))
}

/// Encode a session execute call's calldata
/// This uses the session execution mode for ERC-7579 accounts
#[wasm_bindgen]
pub fn encode_session_execute_call_data(
    target: String,
    value: String,
    data: String,
) -> Result<String, JsValue> {
    use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::session::encode::encode_session_user_operation;

    // Parse target address
    let target_addr = target.parse::<Address>().map_err(|e| {
        JsValue::from_str(&format!("Invalid target address: {}", e))
    })?;

    // Parse value
    let value_u256 = value
        .parse::<U256>()
        .map_err(|e| JsValue::from_str(&format!("Invalid value: {}", e)))?;

    // Parse data (hex string)
    let data_hex = data.trim_start_matches("0x");
    let data_bytes = if data_hex.is_empty() {
        Bytes::default()
    } else {
        let bytes_vec = hex::decode(data_hex).map_err(|e| {
            JsValue::from_str(&format!("Invalid data hex: {}", e))
        })?;
        Bytes::from(bytes_vec)
    };

    let encoded =
        encode_session_user_operation(target_addr, value_u256, data_bytes);

    Ok(format!("0x{}", hex::encode(encoded)))
}

/// Generate a session stub signature for gas estimation
/// Accepts a JSON-encoded SessionSpec and optional timestamp
#[wasm_bindgen]
pub fn generate_session_stub_signature_wasm(
    session_validator: String,
    session_spec_json: String,
    current_timestamp: Option<String>,
) -> Result<String, JsValue> {
    use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::session::encode::generate_session_stub_signature as core_generate_session_stub_signature;
    use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::session::session_lib::session_spec::SessionSpec;

    // Parse validator address
    let validator_addr = session_validator.parse::<Address>().map_err(|e| {
        JsValue::from_str(&format!("Invalid session validator address: {}", e))
    })?;

    // Parse SessionSpec from JSON
    let spec: SessionSpec =
        serde_json::from_str(&session_spec_json).map_err(|e| {
            JsValue::from_str(&format!("Invalid SessionSpec JSON: {}", e))
        })?;

    // Parse timestamp if provided
    let ts_opt = match current_timestamp {
        Some(ts) if !ts.is_empty() => Some(ts.parse::<u64>().map_err(|e| {
            JsValue::from_str(&format!("Invalid timestamp: {}", e))
        })?),
        _ => None,
    };

    let stub =
        core_generate_session_stub_signature(validator_addr, &spec, ts_opt);

    Ok(format!("0x{}", hex::encode(stub)))
}

/// Encode createSession call data for SessionKeyValidator
/// Accepts a JSON-encoded SessionSpec and returns hex-encoded calldata
#[wasm_bindgen]
pub fn encode_create_session_call_data(
    session_spec_json: String,
    proof: String,
) -> Result<String, JsValue> {
    // Parse SessionSpec from JSON
    let spec: SessionSpec =
        serde_json::from_str(&session_spec_json).map_err(|e| {
            JsValue::from_str(&format!("Invalid SessionSpec JSON: {}", e))
        })?;

    let proof_bytes = hex::decode(proof.trim_start_matches("0x"))
        .map_err(|e| JsValue::from_str(&format!("Invalid proof hex: {}", e)))?;

    // Build createSession call and ABI encode
    let create_session_calldata =
        create_session_call_data_core(spec, proof_bytes.into());

    Ok(format!("0x{}", hex::encode(create_session_calldata)))
}

/// Encode sessionState(account, spec) call data for SessionKeyValidator
/// Accepts the smart account address and JSON-encoded SessionSpec and returns hex-encoded calldata
#[wasm_bindgen]
pub fn encode_session_state_call_data(
    account: String,
    session_spec_json: String,
) -> Result<String, JsValue> {
    // Parse account address
    let account_addr = account.parse::<Address>().map_err(|e| {
        JsValue::from_str(&format!("Invalid account address: {}", e))
    })?;

    // Parse SessionSpec from JSON
    let spec: SessionSpec =
        serde_json::from_str(&session_spec_json).map_err(|e| {
            JsValue::from_str(&format!("Invalid SessionSpec JSON: {}", e))
        })?;

    // Build sessionState call and ABI encode
    let call_data = session_state_call_data_core(account_addr, spec);

    Ok(format!("0x{}", hex::encode(call_data)))
}

/// Create a real session signature without relying on system time
/// Accepts private key, validator address, JSON-encoded SessionSpec, userOp hash, and optional timestamp
#[wasm_bindgen]
pub fn session_signature_no_validation_wasm(
    private_key_hex: String,
    session_validator: String,
    session_spec_json: String,
    userop_hash_hex: String,
    current_timestamp: Option<String>,
) -> Result<String, JsValue> {
    use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::session::signature_wasm::session_signature_no_validation as core_session_signature_no_validation;
    use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::session::session_lib::session_spec::SessionSpec;

    // Parse validator address
    let validator_addr = session_validator.parse::<Address>().map_err(|e| {
        JsValue::from_str(&format!("Invalid session validator address: {}", e))
    })?;

    // Parse SessionSpec from JSON
    let spec: SessionSpec =
        serde_json::from_str(&session_spec_json).map_err(|e| {
            JsValue::from_str(&format!("Invalid SessionSpec JSON: {}", e))
        })?;

    // Parse hash
    let hash_hex = userop_hash_hex.trim_start_matches("0x");
    let hash_bytes = hex::decode(hash_hex)
        .map_err(|e| JsValue::from_str(&format!("Invalid hash hex: {}", e)))?;
    if hash_bytes.len() != 32 {
        return Err(JsValue::from_str("Hash must be 32 bytes"));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&hash_bytes);
    let hash = FixedBytes::<32>::from(arr);

    // Parse timestamp if provided
    let ts_opt = match current_timestamp {
        Some(ts) if !ts.is_empty() => Some(ts.parse::<u64>().map_err(|e| {
            JsValue::from_str(&format!("Invalid timestamp: {}", e))
        })?),
        _ => None,
    };

    // Sign
    let signature = core_session_signature_no_validation(
        &private_key_hex,
        validator_addr,
        &spec,
        hash,
        ts_opt,
    )
    .map_err(|e| {
        JsValue::from_str(&format!("Failed to sign session: {}", e))
    })?;

    Ok(format!("0x{}", hex::encode(signature)))
}

/// Compute the keyed nonce (u192) from a session signer address
#[wasm_bindgen]
pub fn keyed_nonce_decimal(session_signer: String) -> Result<String, JsValue> {
    use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::session::send::keyed_nonce as core_keyed_nonce;

    let addr = session_signer
        .parse::<Address>()
        .map_err(|e| JsValue::from_str(&format!("Invalid address: {}", e)))?;

    let u192 = core_keyed_nonce(addr);
    Ok(u192.to_string())
}

/// Sign a message hash with EOA private key
/// The message should already be hashed (e.g., via keccak256)
#[wasm_bindgen]
pub fn sign_eoa_message(
    private_key: String,
    message_hash: String,
) -> Result<String, JsValue> {
    // Parse message hash
    let hash = message_hash.parse::<FixedBytes<32>>().map_err(|e| {
        JsValue::from_str(&format!("Invalid message hash: {}", e))
    })?;

    // Sign the hash
    let signature = eoa_sign_core(&private_key, hash).map_err(|e| {
        JsValue::from_str(&format!("Failed to sign message: {}", e))
    })?;

    Ok(format!("0x{}", hex::encode(&signature)))
}

/// Sign a UserOperation hash with EOA private key
/// This ensures proper s-value normalization for ERC-4337
/// The signature includes the validator address prefix (20 bytes) + ECDSA signature (65 bytes)
#[wasm_bindgen]
pub fn sign_eoa_user_operation_hash(
    user_op_hash: String,
    private_key: String,
    validator_address: String,
) -> Result<String, JsValue> {
    // Parse user operation hash
    let hash = user_op_hash.parse::<FixedBytes<32>>().map_err(|e| {
        JsValue::from_str(&format!("Invalid user operation hash: {}", e))
    })?;

    // Parse validator address
    let validator_addr = validator_address.parse::<Address>().map_err(|e| {
        JsValue::from_str(&format!("Invalid validator address: {}", e))
    })?;

    // Sign the hash with validator address prefix (85 bytes total: 20 + 65)
    let signature = eoa_signature_core(&private_key, validator_addr, hash)
        .map_err(|e| {
            JsValue::from_str(&format!("Failed to sign user operation: {}", e))
        })?;

    Ok(format!("0x{}", hex::encode(&signature)))
}

/// Encode call data for EntryPoint.getUserOpHash() without making network requests.
/// This allows TypeScript/viem to make the actual RPC call, keeping network logic out of Rust SDK.
///
/// # Parameters
/// All UserOperation fields needed to construct PackedUserOperation
///
/// # Returns
/// Hex-encoded call data for calling EntryPoint.getUserOpHash(PackedUserOperation)
#[wasm_bindgen]
pub struct EncodeGetUserOperationHashParams {
    sender: String,
    nonce: String,
    call_data: String,
    call_gas_limit: String,
    verification_gas_limit: String,
    pre_verification_gas: String,
    max_fee_per_gas: String,
    max_priority_fee_per_gas: String,
}

#[wasm_bindgen]
impl EncodeGetUserOperationHashParams {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sender: String,
        nonce: String,
        call_data: String,
        call_gas_limit: String,
        verification_gas_limit: String,
        pre_verification_gas: String,
        max_fee_per_gas: String,
        max_priority_fee_per_gas: String,
    ) -> Self {
        Self {
            sender,
            nonce,
            call_data,
            call_gas_limit,
            verification_gas_limit,
            pre_verification_gas,
            max_fee_per_gas,
            max_priority_fee_per_gas,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn sender(&self) -> String {
        self.sender.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn nonce(&self) -> String {
        self.nonce.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn call_data(&self) -> String {
        self.call_data.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn call_gas_limit(&self) -> String {
        self.call_gas_limit.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn verification_gas_limit(&self) -> String {
        self.verification_gas_limit.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn pre_verification_gas(&self) -> String {
        self.pre_verification_gas.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn max_fee_per_gas(&self) -> String {
        self.max_fee_per_gas.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn max_priority_fee_per_gas(&self) -> String {
        self.max_priority_fee_per_gas.clone()
    }
}

#[wasm_bindgen]
pub fn encode_get_user_operation_hash_call_data(
    params: EncodeGetUserOperationHashParams,
) -> Result<String, JsValue> {
    let EncodeGetUserOperationHashParams {
        sender,
        nonce,
        call_data,
        call_gas_limit,
        verification_gas_limit,
        pre_verification_gas,
        max_fee_per_gas,
        max_priority_fee_per_gas,
    } = params;

    // Parse addresses
    let sender_addr = sender.parse::<Address>().map_err(|e| {
        JsValue::from_str(&format!("Invalid sender address: {}", e))
    })?;

    // Parse numeric fields
    let nonce_u256 = nonce
        .parse::<U256>()
        .map_err(|e| JsValue::from_str(&format!("Invalid nonce: {}", e)))?;

    let call_gas_limit_u256 = call_gas_limit.parse::<U256>().map_err(|e| {
        JsValue::from_str(&format!("Invalid call_gas_limit: {}", e))
    })?;

    let verification_gas_limit_u256 =
        verification_gas_limit.parse::<U256>().map_err(|e| {
            JsValue::from_str(&format!("Invalid verification_gas_limit: {}", e))
        })?;

    let pre_verification_gas_u256 =
        pre_verification_gas.parse::<U256>().map_err(|e| {
            JsValue::from_str(&format!("Invalid pre_verification_gas: {}", e))
        })?;

    let max_fee_per_gas_u256 =
        max_fee_per_gas.parse::<U256>().map_err(|e| {
            JsValue::from_str(&format!("Invalid max_fee_per_gas: {}", e))
        })?;

    let max_priority_fee_per_gas_u256 =
        max_priority_fee_per_gas.parse::<U256>().map_err(|e| {
            JsValue::from_str(&format!(
                "Invalid max_priority_fee_per_gas: {}",
                e
            ))
        })?;

    // Parse call data
    let call_data_hex = call_data.trim_start_matches("0x");
    let call_data_bytes = hex::decode(call_data_hex).map_err(|e| {
        JsValue::from_str(&format!("Invalid call_data hex: {}", e))
    })?;
    let call_data_bytes = Bytes::from(call_data_bytes);

    // Pack gas limits and fees for EntryPoint::PackedUserOperation
    let packed_gas_limits: U256 =
        (verification_gas_limit_u256 << 128) | call_gas_limit_u256;
    let gas_fees: U256 =
        (max_priority_fee_per_gas_u256 << 128) | max_fee_per_gas_u256;

    // Create PackedUserOperation
    let packed_user_op = PackedUserOperation {
        sender: sender_addr,
        nonce: nonce_u256,
        initCode: Bytes::default(),
        callData: call_data_bytes,
        accountGasLimits: packed_gas_limits.to_be_bytes().into(),
        preVerificationGas: pre_verification_gas_u256,
        gasFees: gas_fees.to_be_bytes().into(),
        paymasterAndData: Bytes::default(),
        signature: Bytes::default(), // Empty signature for hash computation
    };

    // Encode the call data
    let encoded = get_user_op_hash_call_data(packed_user_op).map_err(|e| {
        JsValue::from_str(&format!(
            "Failed to get user operation hash call data: {}",
            e
        ))
    })?;

    Ok(format!("0x{}", hex::encode(encoded)))
}

/// Generate a stub signature for gas estimation with passkey validator
/// Returns a properly formatted signature for the validator
#[wasm_bindgen]
pub fn generate_passkey_stub_signature(
    validator_address: String,
) -> Result<String, JsValue> {
    // Parse validator address
    let validator_addr = validator_address.parse::<Address>().map_err(|e| {
        JsValue::from_str(&format!("Invalid validator address: {}", e))
    })?;

    // Generate stub signature
    let signature =
        stub_signature_passkey_core(validator_addr).map_err(|e| {
            JsValue::from_str(&format!(
                "Failed to generate stub signature: {}",
                e
            ))
        })?;

    Ok(format!("0x{}", hex::encode(&signature)))
}

/// Encode a passkey signature for on-chain verification
/// Returns the ABI-encoded signature ready for submission
///
/// # Parameters
/// * `authenticator_data` - Raw authenticator data from WebAuthn
/// * `client_data_json` - Client data JSON string from WebAuthn
/// * `r` - R component of ECDSA signature (must be 32 bytes)
/// * `s` - S component of ECDSA signature (must be 32 bytes)
/// * `credential_id` - The credential ID bytes
///
/// # Returns
/// Hex-encoded ABI signature: (bytes, string, bytes32[2], bytes)
#[wasm_bindgen]
pub fn encode_passkey_signature(
    authenticator_data: &[u8],
    client_data_json: &str,
    r: &[u8],
    s: &[u8],
    credential_id: &[u8],
) -> Result<String, JsValue> {
    use alloy::sol_types::SolType;

    // Validate r and s are 32 bytes
    if r.len() != 32 {
        return Err(JsValue::from_str(&format!(
            "r must be 32 bytes, got {}",
            r.len()
        )));
    }
    if s.len() != 32 {
        return Err(JsValue::from_str(&format!(
            "s must be 32 bytes, got {}",
            s.len()
        )));
    }

    let r_fixed: [u8; 32] = r.try_into().unwrap();
    let s_fixed: [u8; 32] = s.try_into().unwrap();

    // ABI encode using SolType::abi_encode_params to avoid outer tuple wrapper
    // This matches ethers.js AbiCoder.encode() behavior
    // Format: (bytes authenticatorData, string clientDataJSON, bytes32[2] rs, bytes credentialId)
    type SignatureParams = (
        alloy::sol_types::sol_data::Bytes,
        alloy::sol_types::sol_data::String,
        alloy::sol_types::sol_data::FixedArray<
            alloy::sol_types::sol_data::FixedBytes<32>,
            2,
        >,
        alloy::sol_types::sol_data::Bytes,
    );

    let params = (
        authenticator_data.to_vec(),
        client_data_json.to_string(),
        [FixedBytes::from(r_fixed), FixedBytes::from(s_fixed)],
        credential_id.to_vec(),
    );

    let encoded = SignatureParams::abi_encode_params(&params);

    Ok(format!("0x{}", hex::encode(encoded)))
}

/// Generate account ID from optional user ID
/// If user_id is provided, generates deterministic hash
/// If user_id is None, generates random 32-byte ID using getrandom
#[wasm_bindgen]
pub fn generate_account_id(user_id: Option<String>) -> String {
    match user_id {
        Some(id) => {
            // Deterministic: hash the user ID
            let salt = hex::encode(&id);
            let salt_hash = keccak256(salt);
            format!("0x{}", hex::encode(salt_hash))
        }
        None => {
            // Random: generate random 32 bytes using getrandom
            let mut random_bytes = [0u8; 32];
            getrandom::getrandom(&mut random_bytes)
                .expect("Failed to generate random bytes");
            format!("0x{}", hex::encode(random_bytes))
        }
    }
}

/// Encode call data for MSAFactory.deployAccount(bytes32 salt, bytes initData)
/// This is a pure encoding function with NO network calls
///
/// # Parameters
/// * `account_id` - 32-byte hex account ID (salt)
/// * `eoa_signers` - Optional array of EOA signer addresses
/// * `eoa_validator_address` - Required if eoa_signers is provided
/// * `passkey_payload` - Optional passkey payload
/// * `webauthn_validator_address` - Required if passkey_payload is provided
/// # Returns
/// Hex-encoded call data for deployAccount(salt, initData)
#[wasm_bindgen]
pub fn encode_deploy_account_call_data(
    account_id: String,
    eoa_signers: Option<Vec<String>>,
    eoa_validator_address: Option<String>,
    passkey_payload: Option<PasskeyPayload>,
    webauthn_validator_address: Option<String>,
    session_validator_address: Option<String>,
    executor_modules: Option<Vec<String>>,
) -> Result<String, JsValue> {
    use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::{
        deploy::{
            EOASigners as CoreEOASigners, WebAuthNSigner as CoreWebauthNSigner,
        },
        passkey::add::PasskeyPayload as CorePasskeyPayload,
    };

    // Parse account ID
    let account_id_bytes = account_id.trim_start_matches("0x");
    let account_id_fixed =
        FixedBytes::<32>::from_slice(&hex::decode(account_id_bytes).map_err(
            |e| JsValue::from_str(&format!("Invalid account_id hex: {}", e)),
        )?);

    // Parse EOA signers if provided
    let eoa_signers_core = match (eoa_signers, eoa_validator_address) {
        (Some(addresses), Some(validator)) => {
            let mut parsed_addresses = Vec::new();
            for addr_str in addresses {
                match addr_str.parse::<Address>() {
                    Ok(addr) => parsed_addresses.push(addr),
                    Err(e) => {
                        return Err(JsValue::from_str(&format!(
                            "Invalid EOA signer address '{}': {}",
                            addr_str, e
                        )));
                    }
                }
            }

            let validator_addr = validator.parse::<Address>().map_err(|e| {
                JsValue::from_str(&format!(
                    "Invalid eoa_validator_address: {}",
                    e
                ))
            })?;

            Some(CoreEOASigners {
                addresses: parsed_addresses,
                validator_address: validator_addr,
            })
        }
        (None, None) => None,
        _ => {
            return Err(JsValue::from_str(
                "Both eoa_signers and eoa_validator_address must be provided together",
            ));
        }
    };

    // Parse WebAuthn signer if provided
    let webauthn_signer_core = match (
        passkey_payload,
        webauthn_validator_address,
    ) {
        (Some(passkey), Some(validator)) => {
            // Validate coordinate lengths
            if passkey.passkey_x.len() != 32 {
                return Err(JsValue::from_str(&format!(
                    "Invalid passkey X coordinate length: expected 32 bytes, got {}",
                    passkey.passkey_x.len()
                )));
            }
            if passkey.passkey_y.len() != 32 {
                return Err(JsValue::from_str(&format!(
                    "Invalid passkey Y coordinate length: expected 32 bytes, got {}",
                    passkey.passkey_y.len()
                )));
            }

            let passkey_x = FixedBytes::<32>::from_slice(&passkey.passkey_x);
            let passkey_y = FixedBytes::<32>::from_slice(&passkey.passkey_y);

            let validator_addr = validator.parse::<Address>().map_err(|e| {
                JsValue::from_str(&format!(
                    "Invalid webauthn_validator_address: {}",
                    e
                ))
            })?;

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
            return Err(JsValue::from_str(
                "Both passkey_payload and webauthn_validator_address must be provided together",
            ));
        }
    };

    // Parse session validator if provided
    let session_validator_core = match session_validator_address {
        Some(validator) => {
            let validator_addr = validator.parse::<Address>().map_err(|e| {
                JsValue::from_str(&format!(
                    "Invalid session_validator_address: {}",
                    e
                ))
            })?;
            Some(validator_addr)
        }
        None => None,
    };

    // Parse executor modules if provided
    let executor_modules_core = match executor_modules {
        Some(addresses) => {
            let mut parsed_addresses = Vec::new();
            for addr_str in addresses {
                match addr_str.parse::<Address>() {
                    Ok(addr) => {
                        parsed_addresses.push(addr);
                    }
                    Err(e) => {
                        return Err(JsValue::from_str(&format!(
                            "Invalid executor module address '{}': {}",
                            addr_str, e
                        )));
                    }
                }
            }
            Some(parsed_addresses)
        }
        None => {
            None
        }
    };

    // Create init data using the same logic as deploy.rs
    let init_data = create_init_data_for_deployment(
        eoa_signers_core,
        webauthn_signer_core,
        session_validator_core,
        executor_modules_core,
    );

    // Encode the call
    let encoded = deploy_accout_call_data(account_id_fixed, init_data);
    Ok(format!("0x{}", hex::encode(encoded)))
}

/// Encode call data for adding a passkey to a smart account
/// This is a pure encoding function with NO network calls
///
/// # Parameters
/// * `passkey_payload` - Passkey details (credential_id, public_key, domain)
/// * `webauthn_validator_address` - Address of the WebAuthn validator module
///
/// # Returns
/// Hex-encoded call data for MSA.execute() that adds the passkey
#[wasm_bindgen]
pub fn encode_add_passkey_call_data(
    passkey_payload: PasskeyPayload,
    webauthn_validator_address: String,
) -> Result<String, JsValue> {
    // Parse webauthn validator address
    let webauthn_validator =
        webauthn_validator_address.parse::<Address>().map_err(|e| {
            JsValue::from_str(&format!(
                "Invalid webauthn_validator_address: {}",
                e
            ))
        })?;

    // Validate coordinate lengths
    if passkey_payload.passkey_x.len() != 32 {
        return Err(JsValue::from_str(&format!(
            "Invalid passkey X coordinate length: expected 32 bytes, got {}",
            passkey_payload.passkey_x.len()
        )));
    }
    if passkey_payload.passkey_y.len() != 32 {
        return Err(JsValue::from_str(&format!(
            "Invalid passkey Y coordinate length: expected 32 bytes, got {}",
            passkey_payload.passkey_y.len()
        )));
    }

    // Convert PasskeyPayload to core type
    let passkey_x = FixedBytes::<32>::from_slice(&passkey_payload.passkey_x);
    let passkey_y = FixedBytes::<32>::from_slice(&passkey_payload.passkey_y);

    let core_passkey = CorePasskeyPayload {
        credential_id: Bytes::from(passkey_payload.credential_id),
        passkey: [passkey_x, passkey_y],
        origin_domain: passkey_payload.origin_domain,
    };

    // Encode as execute call
    let encoded = add_passkey_call_data_core(core_passkey, webauthn_validator);

    Ok(format!("0x{}", hex::encode(encoded)))
}

/// Helper function to create init data for deployment
/// Mirrors the logic from deploy.rs create_init_data and modules_from_signers
fn create_init_data_for_deployment(
    eoa_signers: Option<CoreEOASigners>,
    webauthn_signer: Option<CoreWebauthNSigner>,
    session_validator: Option<Address>,
    executor_modules: Option<Vec<Address>>,
) -> Bytes {
    sol! {
        struct SignersParams {
            address[] signers;
        }
    }

    // Build modules and data arrays
    let mut modules = Vec::new();
    let mut data = Vec::new();

    // Add EOA signers if provided
    if let Some(signers) = eoa_signers {
        let eoa_signer_encoded =
            SignersParams { signers: signers.addresses.to_vec() }
                .abi_encode_params();
        modules.push(signers.validator_address);
        data.push(Bytes::from(eoa_signer_encoded));
    }

    // Add WebAuthn signer if provided
    if let Some(webauthn) = webauthn_signer {
        let webauthn_signer_encoded = webauthn.passkey.abi_encode_params();
        modules.push(webauthn.validator_address);
        data.push(Bytes::from(webauthn_signer_encoded));
    }

    // Add Session validator if provided (no initialization data needed)
    if let Some(session_val_addr) = session_validator {
        modules.push(session_val_addr);
        data.push(Bytes::new()); // Empty bytes for session validator
    }

    // Add executor modules if provided (no initialization data needed)
    if let Some(executor_addrs) = executor_modules {
        for executor_addr in executor_addrs {
            modules.push(executor_addr);
            data.push(Bytes::new()); // Empty bytes for executor modules
        }
    }

    // Create initializeAccount call

    initialize_account_call_data_core(modules, data)
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
    use super::*;
    use alloy::{
        primitives::{
            Bytes, FixedBytes, U256, Uint, address, bytes, fixed_bytes,
        },
        rpc::types::PackedUserOperation as AlloyPackedUserOperation,
    };
    use zksync_sso_erc4337_core::{
        erc4337::{
            account::{
                erc7579::{
                    calls::encoded_call_data,
                    module::{
                        Module,
                        installed::{
                            IsModuleInstalledParams, is_module_installed,
                        },
                    },
                },
                modular_smart_account::{
                    deploy::{
                        DeployAccountParams, WebAuthNSigner, deploy_account,
                    },
                    passkey::add::PasskeyPayload as CorePasskeyPayload,
                    test_utilities::fund_account_with_default_amount,
                },
            },
            bundler::Bundler,
            entry_point::nonce::get_nonce_with_key as get_nonce_with_key_core,
            signer::test_utils::get_signature_from_js,
            user_operation::hash::user_operation_hash::get_user_operation_hash_entry_point as get_user_operation_hash_entry_point_core,
        },
        utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        },
    };

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

        let webauthn_signer = WebAuthNSigner {
            passkey: passkey_payload.clone(),
            validator_address: webauthn_module,
        };

        let account_address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: None, // No EOA signer
            webauthn_signer: Some(webauthn_signer),
            session_validator: None,
            id: None,
            provider: provider.clone(),
        })
        .await?;

        println!("Account deployed with passkey: {:?}", account_address);

        // Fund the account
        fund_account_with_default_amount(account_address, provider.clone())
            .await?;

        // Verify passkey module is installed
        let is_installed = is_module_installed(IsModuleInstalledParams {
            module: Module::webauthn_validator(webauthn_module),
            account: account_address,
            provider: provider.clone(),
        })
        .await?;
        eyre::ensure!(is_installed, "WebAuthn module is not installed");

        println!("Passkey module verified as installed");

        // ===== TWO-STEP FLOW - MANUAL APPROACH LIKE CORE TEST =====
        // Instead of using send_transaction, manually build UserOp like the passing core test

        println!("\nTesting two-step passkey flow (manual approach)...");

        let call_data: Bytes = encoded_call_data(
            account_address,
            Some(Bytes::default()),
            Some(U256::from(1)),
        );

        let nonce_key = Uint::from(0);
        let nonce = get_nonce_with_key_core(GetNonceWithKeyParams {
            sender: account_address,
            entry_point: entry_point_address,
            key: nonce_key,
            provider: provider.clone(),
        })
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
            signature: stub_sig,
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

        let hash = get_user_operation_hash_entry_point_core(
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
        user_op.signature = full_signature;

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

    /// Test that compares Rust alloy encoding with ethers.js encoding
    /// This helps debug any differences in ABI encoding between the two libraries
    #[test]
    fn test_encode_passkey_signature_matches_ethers() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Deserialize, Serialize)]
        struct TestInput {
            #[serde(rename = "authenticatorData")]
            authenticator_data: Vec<u8>,
            #[serde(rename = "clientDataJSON")]
            client_data_json: String,
            r: Vec<u8>,
            s: Vec<u8>,
            #[serde(rename = "credentialId")]
            credential_id: Vec<u8>,
        }

        #[derive(Debug, Deserialize)]
        struct TestCase {
            name: String,
            input: TestInput,
            expected: String,
        }

        // Load test data generated by ethers.js
        let test_data_json =
            include_str!("../../../../../web/test-data/ethers-test-data.json");
        let test_cases: Vec<TestCase> = serde_json::from_str(test_data_json)
            .expect("Failed to parse test data");

        for test_case in test_cases {
            println!("\n=== Testing case: {} ===", test_case.name);

            // Call our Rust encoding function
            let result = super::encode_passkey_signature(
                &test_case.input.authenticator_data,
                &test_case.input.client_data_json,
                &test_case.input.r,
                &test_case.input.s,
                &test_case.input.credential_id,
            );

            match result {
                Ok(rust_encoded) => {
                    println!(
                        "Expected (ethers): {}",
                        &test_case.expected
                            [..130.min(test_case.expected.len())]
                    );
                    println!(
                        "Got (rust):        {}",
                        &rust_encoded[..130.min(rust_encoded.len())]
                    );

                    let expected_bytes = hex::decode(
                        test_case.expected.trim_start_matches("0x"),
                    )
                    .expect("Failed to decode expected hex");
                    let rust_bytes =
                        hex::decode(rust_encoded.trim_start_matches("0x"))
                            .expect("Failed to decode rust hex");

                    if rust_encoded != test_case.expected {
                        println!("\n❌ MISMATCH!");
                        println!(
                            "Expected length: {} bytes",
                            expected_bytes.len()
                        );
                        println!("Rust length:     {} bytes", rust_bytes.len());

                        // Analyze structure
                        println!("\nExpected structure (first 128 bytes):");
                        for (i, chunk) in
                            expected_bytes.chunks(32).take(4).enumerate()
                        {
                            println!("  Word {}: {}", i, hex::encode(chunk));
                        }

                        println!("\nRust structure (first 128 bytes):");
                        for (i, chunk) in
                            rust_bytes.chunks(32).take(4).enumerate()
                        {
                            println!("  Word {}: {}", i, hex::encode(chunk));
                        }

                        // Compare byte by byte
                        let min_len =
                            expected_bytes.len().min(rust_bytes.len());
                        println!("\nByte-by-byte comparison:");
                        for i in 0..min_len.min(160) {
                            if expected_bytes[i] != rust_bytes[i] {
                                println!(
                                    "  Byte {}: expected 0x{:02x}, got 0x{:02x} ❌",
                                    i, expected_bytes[i], rust_bytes[i]
                                );
                            } else if i % 32 == 0 {
                                println!(
                                    "  Byte {}: 0x{:02x} ✅",
                                    i, expected_bytes[i]
                                );
                            }
                        }

                        panic!(
                            "Encoding mismatch for test case '{}': expected != rust",
                            test_case.name
                        );
                    } else {
                        println!("✅ Match!");
                    }
                }
                Err(e) => {
                    panic!(
                        "Failed to encode test case '{}': {:?}",
                        test_case.name, e
                    );
                }
            }
        }
    }
}
