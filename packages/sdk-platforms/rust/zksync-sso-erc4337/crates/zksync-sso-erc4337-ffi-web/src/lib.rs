use alloy::{
    network::EthereumWallet,
    primitives::{Address, Bytes, FixedBytes, U256, keccak256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol_types::SolEvent,
};
use alloy_rpc_client::RpcClient;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use zksync_sso_erc4337_core::{
    chain::{Chain, id::ChainId},
    config::contracts::Contracts as CoreContracts,
    erc4337::entry_point::version::EntryPointVersion,
    erc4337::account::modular_smart_account::{
        MSAFactory,
        deploy::{EOASigners as CoreEOASigners, MSAInitializeAccount},
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

/// Deploy a smart account using the factory
///
/// # Arguments
/// * `rpc_url` - The RPC endpoint URL
/// * `factory_address` - The address of the MSA factory contract
/// * `user_id` - A unique identifier for the user (will be hashed to create account_id)
/// * `deployer_private_key` - Private key of the account that will pay for deployment (0x-prefixed hex string)
/// * `eoa_signers_addresses` - Optional array of EOA signer addresses (as hex strings)
/// * `eoa_validator_address` - Optional EOA validator module address (required if eoa_signers_addresses is provided)
///
/// # Returns
/// Promise that resolves to the deployed account address as a hex string
#[wasm_bindgen]
pub fn deploy_account(
    rpc_url: String,
    factory_address: String,
    user_id: String,
    deployer_private_key: String,
    eoa_signers_addresses: Option<Vec<String>>,
    eoa_validator_address: Option<String>,
) -> js_sys::Promise {
    future_to_promise(async move {
        console_log!("Starting account deployment...");
        console_log!("  RPC URL: {}", rpc_url);
        console_log!("  Factory: {}", factory_address);
        console_log!("  User ID: {}", user_id);

        // Parse factory address
        let factory_addr = match factory_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!(
                    "Invalid factory address: {}",
                    e
                )));
            }
        };

        // Parse deployer private key
        let deployer_key = match deployer_private_key.trim_start_matches("0x").parse::<PrivateKeySigner>() {
            Ok(signer) => signer,
            Err(e) => {
                return Ok(JsValue::from_str(&format!("Invalid deployer private key: {}", e)));
            }
        };

        let deployer_wallet = EthereumWallet::from(deployer_key);
        console_log!("  Deployer address: {:?}", deployer_wallet.default_signer().address());
        
        // Create transport and provider with wallet
        let transport = WasmHttpTransport::new(rpc_url);
        let client = RpcClient::new(transport.clone(), false);
        let provider = ProviderBuilder::new()
            .wallet(deployer_wallet)
            .connect_client(client);

        // Compute account ID from user ID
        let account_id = compute_account_id(&user_id);
        let account_id_bytes = match hex::decode(account_id.trim_start_matches("0x")) {
            Ok(bytes) => {
                if bytes.len() != 32 {
                    return Ok(JsValue::from_str(&format!("Invalid account ID length: expected 32 bytes, got {}", bytes.len())));
                }
                FixedBytes::<32>::from_slice(&bytes)
            }
            Err(e) => {
                return Ok(JsValue::from_str(&format!("Failed to decode account ID: {}", e)));
            }
        };

        console_log!("  Account ID: 0x{}", hex::encode(account_id_bytes));

        // Parse EOA signers if provided
        let eoa_signers = match (eoa_signers_addresses, eoa_validator_address) {
            (Some(addresses), Some(validator)) => {
                console_log!("  Parsing EOA signers: {} addresses", addresses.len());
                let mut parsed_addresses = Vec::new();
                for addr_str in addresses {
                    match addr_str.parse::<Address>() {
                        Ok(addr) => parsed_addresses.push(addr),
                        Err(e) => {
                            return Ok(JsValue::from_str(&format!("Invalid EOA signer address '{}': {}", addr_str, e)));
                        }
                    }
                }

                let validator_addr = match validator.parse::<Address>() {
                    Ok(addr) => addr,
                    Err(e) => {
                        return Ok(JsValue::from_str(&format!("Invalid validator address: {}", e)));
                    }
                };

                Some(CoreEOASigners {
                    addresses: parsed_addresses,
                    validator_address: validator_addr,
                })
            }
            (None, None) => None,
            _ => {
                return Ok(JsValue::from_str("Both eoa_signers_addresses and eoa_validator_address must be provided together"));
            }
        };

        // Prepare init data
        let (data, modules) = if let Some(signers) = eoa_signers {
            console_log!("  Encoding {} EOA signers", signers.addresses.len());
            use alloy::sol_types::SolValue;
            use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::deploy::SignersParams;
            
            let eoa_signer_encoded = SignersParams { signers: signers.addresses.to_vec() }
                .abi_encode_params()
                .into();
            let modules = vec![signers.validator_address];
            (vec![eoa_signer_encoded], modules)
        } else {
            console_log!("  No EOA signers, deploying empty account");
            (vec![], vec![])
        };
        
        let init_data: Bytes = MSAInitializeAccount::new(modules, data).encode().into();
        console_log!("  Init data length: {} bytes", init_data.len());

        // Create factory instance and deploy
        let factory = MSAFactory::new(factory_addr, provider);

        console_log!("  Calling factory.deployAccount...");
        let deploy_call = factory.deployAccount(account_id_bytes, init_data);

        match deploy_call.send().await {
            Ok(pending_tx) => {
                console_log!("  Transaction sent, waiting for receipt...");
                match pending_tx.get_receipt().await {
                    Ok(receipt) => {
                        console_log!("  Transaction mined!");
                        
                        // Extract account address from AccountCreated event
                        let topic = MSAFactory::AccountCreated::SIGNATURE_HASH;
                        let log = receipt
                            .logs()
                            .iter()
                            .find(|log| log.inner.topics()[0] == topic);
                        
                        if let Some(log) = log {
                            let event = log.inner.topics()[1];
                            let address = Address::from_slice(&event[12..]);
                            let address_str = format!("0x{:x}", address);
                            console_log!("  Deployed account address: {}", address_str);
                            Ok(JsValue::from_str(&address_str))
                        } else {
                            Ok(JsValue::from_str("Account deployed but AccountCreated event not found in logs"))
                        }
                    }
                    Err(e) => {
                        console_log!("  Error getting receipt: {}", e);
                        Ok(JsValue::from_str(&format!("Failed to get transaction receipt: {}", e)))
                    }
                }
            }
            Err(e) => {
                console_log!("  Error sending transaction: {}", e);
                Ok(JsValue::from_str(&format!("Failed to send deployment transaction: {}", e)))
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
    rpc_url: String,
    bundler_url: String,
    account_address: String,
    entry_point_address: String,
    eoa_validator_address: String,
    eoa_private_key: String,
    to_address: String,
    value: String,
    data: Option<String>,
) -> js_sys::Promise {
    future_to_promise(async move {
        console_log!("Starting EOA transaction...");
        console_log!("  Account: {}", account_address);
        console_log!("  To: {}", to_address);
        console_log!("  Value: {}", value);
        console_log!("  Bundler: {}", bundler_url);

        // Parse addresses
        let account = match account_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!("Invalid account address: {}", e)));
            }
        };

        let entry_point = match entry_point_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!("Invalid entry point address: {}", e)));
            }
        };

        let eoa_validator = match eoa_validator_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!("Invalid EOA validator address: {}", e)));
            }
        };

        let to = match to_address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(JsValue::from_str(&format!("Invalid to address: {}", e)));
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
                        return Ok(JsValue::from_str(&format!("Invalid data hex: {}", e)));
                    }
                }
            }
            None => Bytes::default(),
        };

        console_log!("  Parsed addresses and values successfully");

        // Parse EOA private key
        let eoa_key = match eoa_private_key.trim_start_matches("0x").parse::<PrivateKeySigner>() {
            Ok(signer) => signer,
            Err(e) => {
                return Ok(JsValue::from_str(&format!("Invalid EOA private key: {}", e)));
            }
        };

        let eoa_wallet = EthereumWallet::from(eoa_key);
        console_log!("  EOA signer address: {:?}", eoa_wallet.default_signer().address());

        // Create transport and provider
        let transport = WasmHttpTransport::new(rpc_url.clone());
        let client = RpcClient::new(transport.clone(), false);
        let provider = ProviderBuilder::new()
            .wallet(eoa_wallet)
            .connect_client(client);

        console_log!("  Created provider and transport");

        // Create bundler client
        let bundler_client = {
            use zksync_sso_erc4337_core::erc4337::bundler::config::BundlerConfig;
            let config = BundlerConfig::new(bundler_url);
            zksync_sso_erc4337_core::erc4337::bundler::pimlico::client::BundlerClient::new(config)
        };

        console_log!("  Created bundler client");

        // Encode the execution call
        use zksync_sso_erc4337_core::erc4337::account::erc7579::{
            Execution, calls::encode_calls,
        };

        let call = Execution {
            target: to,
            value: value_u256,
            data: data_bytes,
        };

        let calls = vec![call];
        let encoded_calls: Bytes = encode_calls(calls).into();

        console_log!("  Encoded call data");

        // Send transaction using core implementation
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
    let factory_addr: Address = account_factory
        .parse()
        .map_err(|e| JsValue::from_str(&format!("Invalid factory address: {}", e)))?;

    let deploy_wallet_addr: Address = deploy_wallet_address
        .parse()
        .map_err(|e| JsValue::from_str(&format!("Invalid wallet address: {}", e)))?;

    // Parse bytecode hash
    let bytecode_hash_hex = bytecode_hash.strip_prefix("0x").unwrap_or(bytecode_hash);
    let bytecode_hash_bytes = hex::decode(bytecode_hash_hex)
        .map_err(|e| JsValue::from_str(&format!("Invalid bytecode hash: {}", e)))?;
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
