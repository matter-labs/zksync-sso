use alloy::{
    primitives::Address, providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};
use std::str::FromStr;
use url::Url;
use zksync_sso_erc4337_core::{
    chain::{Chain, id::ChainId},
    config::{Config, contracts::Contracts},
    erc4337::{
        client::Client, entry_point::version::EntryPointVersion,
        user_operation::UserOperationV08,
    },
};

#[tokio::test]
async fn test_integration() -> eyre::Result<()> {
    // Test private key (from the JavaScript test)
    let private_key_hex =
        "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";

    // Create signer from private key
    let signer = PrivateKeySigner::from_str(private_key_hex)?;

    // Set up contract addresses (these would typically come from your deployment)
    let contracts = Contracts::new(
        Address::from_str("0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789")?, // EntryPoint v0.7
        Address::from_str("0x9406Cc6185a346906296840746125a0E44976454")?, // AccountFactory placeholder
        Address::from_str("0x9406Cc6185a346906296840746125a0E44976454")?, // EOAValidator placeholder
        Address::from_str("0x9406Cc6185a346906296840746125a0E44976454")?, // WebAuthnValidator placeholder,
        Address::from_str("0x9406Cc6185a346906296840746125a0E44976454")?, // placeholder
    );

    // Anvil
    let rpc_url: Url = "http://localhost:8545".parse()?;

    let chain = Chain::new(
        ChainId::ETHEREUM_MAINNET,
        EntryPointVersion::V08,
        "Mainnet".to_string(),
    );

    // Configure the client
    let config = Config::new(
        rpc_url.clone(),                  // Anvil RPC
        "http://localhost:4337".parse()?, // Alto bundler (typical port)
        chain,
        contracts,
    );

    // Create provider using the simpler HTTP transport
    let provider = ProviderBuilder::new().connect_http(rpc_url);

    // Initialize the client
    let _client = Client::new(config, signer, provider);

    // Create a simple user operation (currently just testing initialization)
    let user_operation = UserOperationV08::default();

    // For now, just test that we can create the client and user operation
    // Full integration test would require:
    // 1. Running anvil locally
    // 2. Deploying contracts
    // 3. Running Alto bundler
    // 4. Sending actual user operations

    println!("Client initialized successfully");
    println!("User operation created: {:?}", user_operation);

    Ok(())
}
