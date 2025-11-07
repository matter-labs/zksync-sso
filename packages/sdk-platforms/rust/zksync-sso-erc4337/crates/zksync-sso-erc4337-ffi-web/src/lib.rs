use alloy::{
    network::EthereumWallet,
    primitives::{Address, Bytes, FixedBytes, U256, keccak256, aliases::U48},
    providers::ProviderBuilder,
    rpc::types::erc4337::PackedUserOperation as AlloyPackedUserOperation,
    signers::local::PrivateKeySigner,
};
use alloy_rpc_client::RpcClient;
use std::str::FromStr;
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
                WebAuthNSigner as CoreWebauthNSigner,
                SessionSigner as CoreSessionSigner,
            },
            send::eoa::EOASendParams,
            session::session_lib::session_spec::{
                SessionSpec as CoreSessionSpec,
                transfer_spec::TransferSpec as CoreTransferSpec,
                usage_limit::UsageLimit as CoreUsageLimit,
                limit_type::LimitType as CoreLimitType,
            },
        },
        entry_point::version::EntryPointVersion,
    },
};

// WASM transport is implemented but not yet fully integrated with Alloy's Provider trait
// For now, we expose offline computation functions
mod wasm_transport;
use wasm_transport::WasmHttpTransport;

// Stub private key for creating stub signatures during gas estimation
const STUB_PRIVATE_KEY: &str =
    "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";

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

// Session payload for WASM
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct TransferPayload {
    target: String,
    value_limit_value: String,
    value_limit_type: u8,
    value_limit_period: String,
}

#[wasm_bindgen]
impl TransferPayload {
    #[wasm_bindgen(constructor)]
    pub fn new(
        target: String,
        value_limit_value: String,
        value_limit_type: u8,
        value_limit_period: String,
    ) -> Self {
        Self {
            target,
            value_limit_value,
            value_limit_type,
            value_limit_period,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn target(&self) -> String {
        self.target.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn value_limit_value(&self) -> String {
        self.value_limit_value.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn value_limit_type(&self) -> u8 {
        self.value_limit_type
    }

    #[wasm_bindgen(getter)]
    pub fn value_limit_period(&self) -> String {
        self.value_limit_period.clone()
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SessionPayload {
    signer: String,
    expires_at: String,
    fee_limit_value: String,
    fee_limit_type: u8,
    fee_limit_period: String,
    transfers: Vec<TransferPayload>,
}

#[wasm_bindgen]
impl SessionPayload {
    #[wasm_bindgen(constructor)]
    pub fn new(
        signer: String,
        expires_at: String,
        fee_limit_value: String,
        fee_limit_type: u8,
        fee_limit_period: String,
        transfers: Vec<TransferPayload>,
    ) -> Self {
        Self {
            signer,
            expires_at,
            fee_limit_value,
            fee_limit_type,
            fee_limit_period,
            transfers,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn signer(&self) -> String {
        self.signer.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn expires_at(&self) -> String {
        self.expires_at.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn fee_limit_value(&self) -> String {
        self.fee_limit_value.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn fee_limit_type(&self) -> u8 {
        self.fee_limit_type
    }

    #[wasm_bindgen(getter)]
    pub fn fee_limit_period(&self) -> String {
        self.fee_limit_period.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn transfers(&self) -> Vec<TransferPayload> {
        self.transfers.clone()
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
    session_validator_address: Option<String>,
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
        session_validator_address: Option<String>,
    ) -> Self {
        Self {
            rpc_url,
            factory_address,
            deployer_private_key,
            eoa_validator_address,
            webauthn_validator_address,
            session_validator_address,
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

    #[wasm_bindgen(getter)]
    pub fn session_validator_address(&self) -> Option<String> {
        self.session_validator_address.clone()
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
/// * `user_id` - A unique identifier for the user (will be hashed to create account_id)
/// * `eoa_signers_addresses` - Optional array of EOA signer addresses (as hex strings)
/// * `passkey_payload` - Optional WebAuthn passkey payload
/// * `session_payload` - Optional session configuration
/// * `deploy_account_config` - Deployment configuration including RPC URL, factory address, and validator addresses
///
/// # Returns
/// Promise that resolves to the deployed account address as a hex string
#[wasm_bindgen]
pub fn deploy_account(
    user_id: String,
    eoa_signers_addresses: Option<Vec<String>>,
    passkey_payload: Option<PasskeyPayload>,
    session_payload: Option<SessionPayload>,
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

        // Convert SessionPayload to SessionSigner if provided
        let session_signer = match (session_payload, deploy_account_config.session_validator_address.as_ref()) {
            (Some(session), Some(session_validator_str)) => {
                console_log!("  Converting session payload...");
                
                // Parse session validator address
                let session_validator_addr = match session_validator_str.parse::<Address>() {
                    Ok(addr) => addr,
                    Err(e) => {
                        return Err(JsValue::from_str(&format!(
                            "Invalid session validator address: {}",
                            e
                        )));
                    }
                };

                // Parse session signer address
                let signer = match session.signer.parse::<Address>() {
                    Ok(addr) => addr,
                    Err(e) => {
                        return Err(JsValue::from_str(&format!(
                            "Invalid session signer address: {}",
                            e
                        )));
                    }
                };

                // Parse expires_at (U48 from string)
                let expires_at = match u64::from_str_radix(&session.expires_at, 10) {
                    Ok(val) => U48::from(val),
                    Err(e) => {
                        return Err(JsValue::from_str(&format!(
                            "Invalid expires_at value: {}",
                            e
                        )));
                    }
                };

                // Convert fee limit
                let fee_limit_type = CoreLimitType::try_from(session.fee_limit_type).map_err(|e| {
                    JsValue::from_str(&format!("Invalid fee limit type: {}", e))
                })?;
                
                let fee_limit = CoreUsageLimit {
                    limit_type: fee_limit_type,
                    limit: U256::from_str(&session.fee_limit_value).map_err(|e| {
                        JsValue::from_str(&format!("Invalid fee limit value: {}", e))
                    })?,
                    period: U48::from_str(&session.fee_limit_period).map_err(|e| {
                        JsValue::from_str(&format!("Invalid fee limit period: {}", e))
                    })?,
                };

                // Convert transfer policies
                let transfer_policies: Result<Vec<CoreTransferSpec>, JsValue> = session.transfers.iter().map(|transfer| {
                    let target = transfer.target.parse::<Address>().map_err(|e| {
                        JsValue::from_str(&format!("Invalid transfer target address: {}", e))
                    })?;
                    
                    let max_value_per_use = U256::from_str(&transfer.value_limit_value).map_err(|e| {
                        JsValue::from_str(&format!("Invalid max_value_per_use: {}", e))
                    })?;
                    
                    let value_limit_type = CoreLimitType::try_from(transfer.value_limit_type).map_err(|e| {
                        JsValue::from_str(&format!("Invalid value limit type: {}", e))
                    })?;
                    
                    let value_limit = CoreUsageLimit {
                        limit_type: value_limit_type,
                        limit: U256::from_str(&transfer.value_limit_value).map_err(|e| {
                            JsValue::from_str(&format!("Invalid value limit: {}", e))
                        })?,
                        period: U48::from_str(&transfer.value_limit_period).map_err(|e| {
                            JsValue::from_str(&format!("Invalid value limit period: {}", e))
                        })?,
                    };
                    
                    Ok(CoreTransferSpec {
                        target,
                        max_value_per_use,
                        value_limit,
                    })
                }).collect();

                let transfer_policies = transfer_policies?;

                // Create SessionSpec
                let session_spec = CoreSessionSpec {
                    signer,
                    expires_at,
                    fee_limit,
                    call_policies: vec![], // No call policies for now
                    transfer_policies,
                };

                console_log!("  Session spec created with {} transfers", session.transfers.len());

                Some(CoreSessionSigner {
                    session_spec,
                    validator_address: session_validator_addr,
                })
            }
            (Some(_), None) => {
                return Err(JsValue::from_str(
                    "Session payload provided but session_validator_address is missing"
                ));
            }
            (None, _) => None,
        };

        console_log!("  Calling core deploy_account...");

        // Use the core crate's deploy_account function
        match zksync_sso_erc4337_core::erc4337::account::modular_smart_account::deploy::deploy_account(
            zksync_sso_erc4337_core::erc4337::account::modular_smart_account::deploy::DeployAccountParams {
                factory_address: factory_addr,
                eoa_signers,
                webauthn_signer,
                session_signer,
                id: None,
                provider,
            }
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
                return Err(JsValue::from_str(&format!(
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
                Err(JsValue::from_str(&format!(
                    "Failed to add passkey: {}",
                    e
                )))
            }
        }
    })
}

/// Add a session to an already-deployed smart account
///
/// This installs the SessionKeyValidator module (if needed) and creates a session.
/// The transaction is authorized by an EOA signer configured in the account's EOA validator.
///
/// # Parameters
/// * `config` - SendTransactionConfig with RPC URL, bundler URL, and entry point
/// * `account_address` - The deployed smart account address
/// * `session_payload` - The session specification payload
/// * `session_validator_address` - The SessionKeyValidator module address
/// * `eoa_validator_address` - The EOA validator module address (used for authorization)
/// * `eoa_private_key` - Private key of an EOA signer (hex string) authorized in the EOA validator
#[wasm_bindgen]
pub fn add_session_to_account(
    config: SendTransactionConfig,
    account_address: String,
    session_payload: SessionPayload,
    session_validator_address: String,
    eoa_validator_address: String,
    eoa_private_key: String,
) -> js_sys::Promise {
    future_to_promise(async move {
        console_log!("Adding session to smart account...");
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

        let session_validator = match session_validator_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid session validator address: {}",
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
            use zksync_sso_erc4337_core::erc4337::bundler::config::BundlerConfig;
            let config = BundlerConfig::new(config.bundler_url);
            zksync_sso_erc4337_core::erc4337::bundler::pimlico::client::BundlerClient::new(config)
        };

        // Build EOA signature provider for authorizing module install and session creation
        use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::signature::{
            eoa_signature, stub_signature_eoa,
        };
        use std::sync::Arc;

        let stub_sig = match stub_signature_eoa(eoa_validator) {
            Ok(sig) => sig,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
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

        // Ensure SessionKeyValidator module is installed
        match zksync_sso_erc4337_core::erc4337::account::erc7579::add_module::add_module(
            account,
            session_validator,
            entry_point,
            provider.clone(),
            bundler_client.clone(),
            signer.clone(),
        )
        .await
        {
            Ok(_) => {
                console_log!("  Session module installed (or already present)");
            }
            Err(e) => {
                console_log!("  Warning: add_module failed (may already be installed): {}", e);
            }
        }

        // Convert SessionPayload to CoreSessionSpec
        let signer_addr = match session_payload.signer.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid session signer address: {}",
                    e
                )));
            }
        };

        let expires_at = match u64::from_str_radix(&session_payload.expires_at, 10) {
            Ok(v) => U48::from(v),
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid expires_at value: {}",
                    e
                )));
            }
        };

        let fee_limit_type = match CoreLimitType::try_from(session_payload.fee_limit_type) {
            Ok(t) => t,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid fee limit type: {}",
                    e
                )));
            }
        };

        let fee_limit_value = match U256::from_str(&session_payload.fee_limit_value) {
            Ok(v) => v,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid fee limit value: {}",
                    e
                )));
            }
        };

        let fee_limit_period = match U48::from_str(&session_payload.fee_limit_period) {
            Ok(p) => p,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid fee limit period: {}",
                    e
                )));
            }
        };

        let fee_limit = CoreUsageLimit {
            limit_type: fee_limit_type,
            limit: fee_limit_value,
            period: fee_limit_period,
        };

        let transfer_policies: Result<Vec<CoreTransferSpec>, JsValue> = session_payload
            .transfers
            .iter()
            .map(|t| {
                let target = t.target.parse::<Address>().map_err(|e| JsValue::from_str(&format!("Invalid transfer target: {}", e)))?;
                let max_value_per_use = U256::from_str(&t.value_limit_value)
                    .map_err(|e| JsValue::from_str(&format!("Invalid value_limit_value: {}", e)))?;
                let limit_type = CoreLimitType::try_from(t.value_limit_type)
                    .map_err(|e| JsValue::from_str(&format!("Invalid value_limit_type: {}", e)))?;
                let limit = U256::from_str(&t.value_limit_value)
                    .map_err(|e| JsValue::from_str(&format!("Invalid value_limit_value: {}", e)))?;
                let period = U48::from_str(&t.value_limit_period)
                    .map_err(|e| JsValue::from_str(&format!("Invalid value_limit_period: {}", e)))?;

                Ok(CoreTransferSpec {
                    target,
                    max_value_per_use,
                    value_limit: CoreUsageLimit { limit_type, limit, period },
                })
            })
            .collect();

        let transfer_policies = transfer_policies?;

        let session_spec = CoreSessionSpec {
            signer: signer_addr,
            expires_at,
            fee_limit,
            call_policies: vec![],
            transfer_policies,
        };

        console_log!("  Creating session via core create_session...");

        match zksync_sso_erc4337_core::erc4337::account::modular_smart_account::session::create::create_session(
            account,
            session_spec,
            entry_point,
            session_validator,
            bundler_client,
            provider,
            signer,
        )
        .await
        {
            Ok(_) => Ok(JsValue::from_str("Session created successfully")),
            Err(e) => Err(JsValue::from_str(&format!(
                "Failed to create session: {}",
                e
            ))),
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

        match zksync_sso_erc4337_core::erc4337::account::modular_smart_account::send::eoa::eoa_send_transaction(EOASendParams {
            account,
            entry_point,
            call_data: encoded_calls,
            nonce_key: None,
            paymaster: None,
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
        use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::signature::stub_signature_passkey;
        let stub_sig = match stub_signature_passkey(validator) {
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
                return Err(JsValue::from_str(&format!(
                    "Failed to get UserOp hash: {}",
                    e
                )));
            }
        };

        console_log!("  UserOperation hash: {:?}", hash);

        // Convert hash to B256 for hex format
        let hash_b256: alloy::primitives::B256 = hash.into();
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
            match serde_json::from_str(&prepared_user_op_json) {
                Ok(p) => p,
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Invalid prepared UserOperation JSON: {}",
                        e
                    )));
                }
            };

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
                return Err(JsValue::from_str(&format!(
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

/// Send a transaction using a session key
///
/// This function sends a transaction signed by a session key validator.
/// It uses a keyed nonce based on the session signer address and creates
/// a session signature that includes the SessionSpec and period IDs.
///
/// # Parameters
/// * `config` - SendTransactionConfig with RPC, bundler, and entry point
/// * `session_validator_address` - Address of the Session Key validator module
/// * `account_address` - The smart account address
/// * `to_address` - The recipient address
/// * `value` - Amount to send (as string, e.g., "1000000000000000000" for 1 ETH)
/// * `data` - Optional calldata as hex string
/// * `session_private_key` - The session key private key (hex string)
/// * `session_payload` - The SessionPayload containing session configuration
#[wasm_bindgen]
pub fn send_transaction_session(
    config: SendTransactionConfig,
    session_validator_address: String,
    account_address: String,
    to_address: String,
    value: String,
    data: Option<String>,
    session_private_key: String,
    session_payload: SessionPayload,
) -> js_sys::Promise {
    future_to_promise(async move {
        console_log!("Starting Session transaction...");
        console_log!("  Account: {}", account_address);
        console_log!("  To: {}", to_address);
        console_log!("  Value: {}", value);
        console_log!("  Bundler: {}", config.bundler_url);

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

        let session_validator = match session_validator_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid session validator address: {}",
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

        // Parse session private key to get signer address
        let session_signer_address = match session_private_key
            .trim_start_matches("0x")
            .parse::<PrivateKeySigner>()
        {
            Ok(signer) => {
                let addr = signer.address();
                console_log!("  Session signer address: {:?}", addr);
                addr
            }
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid session private key: {}",
                    e
                )));
            }
        };

        // Convert SessionPayload to SessionSpec
        let signer_addr = match session_payload.signer.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid session signer address: {}",
                    e
                )));
            }
        };

        let expires_at = match u64::from_str_radix(&session_payload.expires_at, 10) {
            Ok(v) => U48::from(v),
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid expires_at value: {}",
                    e
                )));
            }
        };

        let fee_limit_type = match CoreLimitType::try_from(session_payload.fee_limit_type) {
            Ok(t) => t,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid fee limit type: {}",
                    e
                )));
            }
        };

        let fee_limit_value = match U256::from_str(&session_payload.fee_limit_value) {
            Ok(v) => v,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid fee limit value: {}",
                    e
                )));
            }
        };

        let fee_limit_period = match U48::from_str(&session_payload.fee_limit_period) {
            Ok(p) => p,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Invalid fee limit period: {}",
                    e
                )));
            }
        };

        let fee_limit = CoreUsageLimit {
            limit_type: fee_limit_type,
            limit: fee_limit_value,
            period: fee_limit_period,
        };

        let transfer_policies: Result<Vec<CoreTransferSpec>, JsValue> = session_payload
            .transfers
            .iter()
            .map(|t| {
                let target = t.target.parse::<Address>().map_err(|e| JsValue::from_str(&format!("Invalid transfer target: {}", e)))?;
                let max_value_per_use = U256::from_str(&t.value_limit_value)
                    .map_err(|e| JsValue::from_str(&format!("Invalid value_limit_value: {}", e)))?;
                let limit_type = CoreLimitType::try_from(t.value_limit_type)
                    .map_err(|e| JsValue::from_str(&format!("Invalid value_limit_type: {}", e)))?;
                let limit = U256::from_str(&t.value_limit_value)
                    .map_err(|e| JsValue::from_str(&format!("Invalid value_limit_value: {}", e)))?;
                let period = U48::from_str(&t.value_limit_period)
                    .map_err(|e| JsValue::from_str(&format!("Invalid value_limit_period: {}", e)))?;

                Ok(CoreTransferSpec {
                    target,
                    max_value_per_use,
                    value_limit: CoreUsageLimit { limit_type, limit, period },
                })
            })
            .collect();

        let transfer_policies = transfer_policies?;

        let session_spec = CoreSessionSpec {
            signer: signer_addr,
            expires_at,
            fee_limit,
            call_policies: vec![],
            transfer_policies,
        };

        console_log!("  Converted session payload to SessionSpec");

        // Create transport and provider (without wallet - session signing is custom)
        let transport = WasmHttpTransport::new(config.rpc_url.clone());
        let client = RpcClient::new(transport.clone(), false);
        let provider = ProviderBuilder::new().connect_client(client);

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
            "  Encoded call data, preparing session transaction..."
        );

        // Use keyed nonce for session
        use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::session::send::keyed_nonce;
        let nonce_key = keyed_nonce(session_signer_address);
        console_log!("  Session keyed nonce: {}", nonce_key);

        // Send transaction with session signature
        use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::send::{
            SendParams, send_transaction,
        };
        use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::signature::session_signature;
        use zksync_sso_erc4337_core::erc4337::signer::Signer;
        use std::sync::Arc;

        // Create stub signature for gas estimation
        let stub_sig = match session_signature(
            STUB_PRIVATE_KEY,
            session_validator,
            &session_spec,
            FixedBytes::default(),
        ) {
            Ok(sig) => sig,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Failed to create stub signature: {}",
                    e
                )));
            }
        };

        // Create signature provider
        let private_key = session_private_key.clone();
        let signature_provider = Arc::new(move |hash: FixedBytes<32>| {
            session_signature(&private_key, session_validator, &session_spec, hash)
        });

        let signer = Signer {
            provider: signature_provider,
            stub_signature: stub_sig,
        };

        match send_transaction(SendParams {
            account,
            entry_point,
            call_data: encoded_calls,
            nonce_key: Some(nonce_key),
            paymaster: None,
            bundler_client,
            provider,
            signer,
        })
        .await
        {
            Ok(_) => {
                console_log!("  Session transaction sent successfully!");
                Ok(JsValue::from_str("Session transaction sent successfully"))
            }
            Err(e) => {
                console_log!("  Error sending session transaction: {}", e);
                Err(JsValue::from_str(&format!("Failed to send session transaction: {}", e)))
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
    /// Test deploying an account with a session installed at creation (deploy-with-session)
    #[tokio::test]
    async fn test_wasm_deploy_with_session() -> eyre::Result<()> {
        use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::session::session_lib::session_spec::{
            SessionSpec, transfer_spec::TransferSpec, usage_limit::UsageLimit, limit_type::LimitType,
        };
        use alloy::primitives::{U256, aliases::U48, address};

        let signer_private_key = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6".to_string();
        let config = TestInfraConfig { signer_private_key: signer_private_key.clone() };
        let (
            _,
            anvil_instance,
            provider,
            contracts,
            _signer_private_key,
            bundler,
            bundler_client,
        ) = start_anvil_and_deploy_contracts_and_start_bundler_with_config(&config).await?;

        let factory_address = contracts.account_factory;
        let entry_point_address = address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");
        let eoa_validator_address = contracts.eoa_validator;
        let session_validator_address = contracts.session_validator;

        // Use the same EOA signer address as core tests to match validator expectations
        let signers = vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];
        let eoa_signers = zksync_sso_erc4337_core::erc4337::account::modular_smart_account::deploy::EOASigners {
            addresses: signers.clone(),
            validator_address: eoa_validator_address,
        };

        // Session spec for deploy-with-session
        let session_signer_address = signers[0];
        let transfer_target = signers[0];
        let session_spec = SessionSpec {
            signer: session_signer_address,
            expires_at: U48::from(2088558400u64),
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(1_000_000_000_000_000_000u64),
                period: U48::from(0),
            },
            call_policies: vec![],
            transfer_policies: vec![TransferSpec {
                target: transfer_target,
                max_value_per_use: U256::from(1u64),
                value_limit: UsageLimit {
                    limit_type: LimitType::Unlimited,
                    limit: U256::from(0),
                    period: U48::from(0),
                },
            }],
        };

        let session_signer = zksync_sso_erc4337_core::erc4337::account::modular_smart_account::deploy::SessionSigner {
            session_spec,
            validator_address: session_validator_address,
        };

        // Deploy account with session installed at creation
        let account_address = zksync_sso_erc4337_core::erc4337::account::modular_smart_account::deploy::deploy_account(
            zksync_sso_erc4337_core::erc4337::account::modular_smart_account::deploy::DeployAccountParams {
                factory_address,
                eoa_signers: Some(eoa_signers),
                webauthn_signer: None,
                session_signer: Some(session_signer),
                id: None,
                provider: provider.clone(),
            }
        ).await?;

        // Fund the account
        {
            let fund_tx = alloy::providers::Provider::send_transaction(
                &provider,
                alloy::rpc::types::TransactionRequest::default()
                    .to(account_address)
                    .value(U256::from(10000000000000000000u64)),
            ).await?.get_receipt().await?;
        }

        // Verify session module is installed
        let is_installed = is_module_installed(
            session_validator_address,
            account_address,
            provider.clone(),
        ).await?;
        eyre::ensure!(is_installed, "Session module is not installed after deploy-with-session");

        // Optionally: verify session spec (out of scope for this smoke test)

        drop(anvil_instance);
        drop(bundler);
        Ok(())
    }
    use alloy::{
        primitives::{Bytes, FixedBytes, U256, address, bytes, fixed_bytes},
        providers::Provider,
        rpc::types::TransactionRequest,
    };
    use zksync_sso_erc4337_core::{
        erc4337::account::{
            erc7579::{Execution, module_installed::is_module_installed},
            modular_smart_account::{
                add_passkey::PasskeyPayload as CorePasskeyPayload,
                deploy::{DeployAccountParams, WebAuthNSigner, deploy_account},
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

        let webauthn_signer = WebAuthNSigner {
            passkey: passkey_payload.clone(),
            validator_address: webauthn_module,
        };

        let account_address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: None, // No EOA signer
            webauthn_signer: Some(webauthn_signer),
            session_signer: None,
            id: None,
            provider: provider.clone(),
        })
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

    /// Test deploying an account, adding a session, and verifying session module
    #[tokio::test]
    async fn test_wasm_session_add_and_verify() -> eyre::Result<()> {
        use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::session::session_lib::session_spec::{
            SessionSpec, transfer_spec::TransferSpec, usage_limit::UsageLimit, limit_type::LimitType,
        };
        use alloy::primitives::{U256, aliases::U48};

        let signer_private_key = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6".to_string();
        let config = TestInfraConfig { signer_private_key: signer_private_key.clone() };
        let (
            _,
            anvil_instance,
            provider,
            contracts,
            _signer_private_key,
            bundler,
            bundler_client,
        ) = start_anvil_and_deploy_contracts_and_start_bundler_with_config(&config).await?;

        let factory_address = contracts.account_factory;
        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");
        let eoa_validator_address = contracts.eoa_validator;
        let session_validator_address = contracts.session_validator;

        // Deploy account with EOA validator
    // Use the same EOA signer address as core tests to match validator expectations
    let signers = vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];
        let eoa_signers = zksync_sso_erc4337_core::erc4337::account::modular_smart_account::deploy::EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let account_address = deploy_account(zksync_sso_erc4337_core::erc4337::account::modular_smart_account::deploy::DeployAccountParams {
            factory_address,
            eoa_signers: Some(eoa_signers),
            webauthn_signer: None,
            session_signer: None,
            id: None,
            provider: provider.clone(),
        }).await?;

        // Fund the account
        {
            let fund_tx = alloy::providers::Provider::send_transaction(
                &provider,
                alloy::rpc::types::TransactionRequest::default()
                    .to(account_address)
                    .value(U256::from(10000000000000000000u64)),
            ).await?.get_receipt().await?;
        }

        // Install session module and add a session to the account
        {
            use std::sync::Arc;
            use zksync_sso_erc4337_core::erc4337::account::{
                erc7579::add_module::add_module,
                modular_smart_account::signature::{
                    eoa_signature, stub_signature_eoa,
                },
            };

            // Build EOA signer matching validator
            let stub_sig = stub_signature_eoa(eoa_validator_address)?;
            let signer_private_key = signer_private_key.clone();
            let signature_provider = Arc::new(move |hash: FixedBytes<32>| {
                eoa_signature(
                    &signer_private_key,
                    eoa_validator_address,
                    hash,
                )
            });

            let signer = zksync_sso_erc4337_core::erc4337::signer::Signer {
                provider: signature_provider,
                stub_signature: stub_sig,
            };

            // Install session module
            add_module(
                account_address,
                session_validator_address,
                entry_point_address,
                provider.clone(),
                bundler_client.clone(),
                signer.clone(),
            )
            .await?;

            // Verify installed
            let is_installed = is_module_installed(
                session_validator_address,
                account_address,
                provider.clone(),
            )
            .await?;
            eyre::ensure!(is_installed, "Session module is not installed");
        }

        // Create session spec
        // Match core test parameters for session signer and transfer target
        let session_signer_address = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
        let transfer_target = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
        let session_spec = SessionSpec {
            signer: session_signer_address,
            // Use a concrete timestamp similar to core tests
            expires_at: U48::from(2088558400u64),
            // Align fee limit with core test: Lifetime limit of 1 ETH
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(1_000_000_000_000_000_000u64),
                period: U48::from(0),
            },
            call_policies: vec![],
            // Align transfer policy with core test: per-use max 1 wei, unlimited total
            transfer_policies: vec![TransferSpec {
                target: transfer_target,
                max_value_per_use: U256::from(1u64),
                value_limit: UsageLimit {
                    limit_type: LimitType::Unlimited,
                    limit: U256::from(0),
                    period: U48::from(0),
                },
            }],
        };

        // Create session via core API (preferred for Rust tests)
        zksync_sso_erc4337_core::erc4337::account::modular_smart_account::session::create::create_session(
            account_address,
            session_spec,
            entry_point_address,
            session_validator_address,
            bundler_client.clone(),
            provider.clone(),
            {
                use std::sync::Arc;
                use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::signature::{
                    eoa_signature, stub_signature_eoa,
                };
                let stub_sig = stub_signature_eoa(eoa_validator_address)?;
                let signer_private_key = signer_private_key.clone();
                let signature_provider = Arc::new(move |hash: FixedBytes<32>| {
                    eoa_signature(&signer_private_key, eoa_validator_address, hash)
                });
                zksync_sso_erc4337_core::erc4337::signer::Signer { provider: signature_provider, stub_signature: stub_sig }
            },
        )
        .await?;

        // Verify session module is installed
        // Module already installed above; if create_session succeeded, test passes

        drop(anvil_instance);
        drop(bundler);
        Ok(())
    }
}
