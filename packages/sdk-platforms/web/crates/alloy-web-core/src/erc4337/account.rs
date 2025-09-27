use crate::erc4337::contracts::{SimpleAccount, SimpleAccountFactory};
use alloy::network::EthereumWallet;
use alloy::primitives::{Address, FixedBytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use tracing::info;

pub struct AccountDeployment {
    pub factory_address: Address,
    pub account_address: Address,
    pub owner: Address,
    pub entry_point: Address,
}

pub struct DeploymentConfig {
    pub rpc_url: String,
    pub private_key: String,
    pub factory_address: Option<String>,
    pub entry_point_address: Option<String>,
    pub owner_address: Option<String>,
    pub salt: Option<u64>,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8545".to_string(),
            // Default Anvil private key
            private_key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
                .to_string(),
            factory_address: None,
            entry_point_address: None,
            owner_address: None,
            salt: Some(0),
        }
    }
}

pub async fn deploy_simple_account(config: DeploymentConfig) -> Result<AccountDeployment, String> {
    info!("Deploying SimpleAccount with config");

    // Parse private key
    let signer = PrivateKeySigner::from_bytes(&FixedBytes::<32>::from_slice(
        &alloy::primitives::hex::decode(config.private_key.trim_start_matches("0x"))
            .map_err(|e| format!("Invalid private key: {e}"))?,
    ))
    .map_err(|e| format!("Failed to create signer: {e}"))?;

    let wallet = EthereumWallet::from(signer.clone());

    // Set up provider
    let provider = ProviderBuilder::new().wallet(wallet).connect_http(
        config
            .rpc_url
            .parse()
            .map_err(|e| format!("Invalid RPC URL: {e}"))?,
    );

    // Determine owner address
    let owner = if let Some(owner_str) = config.owner_address {
        owner_str
            .parse::<Address>()
            .map_err(|e| format!("Invalid owner address: {e}"))?
    } else {
        signer.address()
    };
    let salt = U256::from(config.salt.unwrap_or(0));

    info!("Owner address: {}", owner);

    // Check if factory exists, if not we need to inform the user
    let factory_address = config
        .factory_address
        .ok_or_else(|| {
            "Factory address not provided. Please deploy the contracts first using forge scripts."
                .to_string()
        })
        .and_then(|s| {
            s.parse::<Address>()
                .map_err(|e| format!("Invalid factory address: {e}"))
        })?;

    let _entry_point_address = config.entry_point_address
        .ok_or_else(|| "EntryPoint address not provided. Please deploy the contracts first using forge scripts.".to_string())
        .and_then(|s| s.parse::<Address>()
            .map_err(|e| format!("Invalid entry point address: {e}")))?;

    // Get factory contract instance
    let factory = SimpleAccountFactory::new(factory_address, &provider);

    // Get the counterfactual address
    let counterfactual_address = factory
        .getAddress(owner, salt)
        .call()
        .await
        .map_err(|e| format!("Failed to get counterfactual address: {e}"))?;

    info!("Counterfactual address: {}", counterfactual_address);

    // Check if account already exists
    let code = provider
        .get_code_at(counterfactual_address)
        .await
        .map_err(|e| format!("Failed to check account code: {e}"))?;

    let account_address = if !code.is_empty() {
        info!("Account already deployed at {}", counterfactual_address);
        counterfactual_address
    } else {
        info!("Deploying new account...");

        // Deploy the account
        let tx = factory
            .createAccount(owner, salt)
            .send()
            .await
            .map_err(|e| format!("Failed to send deployment transaction: {e}"))?;

        let receipt = tx
            .get_receipt()
            .await
            .map_err(|e| format!("Failed to get transaction receipt: {e}"))?;

        info!("Account deployed in tx: {}", receipt.transaction_hash);

        // Get the deployed address from the receipt
        counterfactual_address
    };

    // Verify the deployment
    let account = SimpleAccount::new(account_address, &provider);
    let deployed_owner = account
        .owner()
        .call()
        .await
        .map_err(|e| format!("Failed to get account owner: {e}"))?;

    if deployed_owner != owner {
        return Err(format!(
            "Account owner mismatch. Expected: {owner}, Got: {deployed_owner}"
        ));
    }

    let deployed_entry_point = account
        .entryPoint()
        .call()
        .await
        .map_err(|e| format!("Failed to get account entry point: {e}"))?;

    info!("Account deployed successfully!");
    info!("  Factory: {}", factory_address);
    info!("  Account: {}", account_address);
    info!("  Owner: {}", owner);
    info!("  EntryPoint: {}", deployed_entry_point);

    Ok(AccountDeployment {
        factory_address,
        account_address,
        owner,
        entry_point: deployed_entry_point,
    })
}

/// Compute the counterfactual address for a SimpleAccount
pub fn compute_account_address(factory: Address, owner: Address, salt: U256) -> Address {
    use alloy::primitives::keccak256;
    use alloy::sol_types::SolCall;

    // Encode the createAccount call
    let call = SimpleAccountFactory::createAccountCall { owner, salt };
    let init_calldata = call.abi_encode();

    // The init code is factory address + calldata
    let mut init_code = Vec::new();
    init_code.extend_from_slice(factory.as_slice());
    init_code.extend_from_slice(&init_calldata);

    // Compute CREATE2 address
    // address = keccak256(0xff ++ factory ++ salt ++ keccak256(init_code))[12:]
    let init_code_hash = keccak256(&init_code);

    let mut to_hash = Vec::new();
    to_hash.push(0xff);
    to_hash.extend_from_slice(factory.as_slice());
    to_hash.extend_from_slice(&salt.to_be_bytes::<32>());
    to_hash.extend_from_slice(init_code_hash.as_slice());

    let hash = keccak256(&to_hash);
    Address::from_slice(&hash[12..])
}

/// Get the init code for deploying a SimpleAccount
pub fn get_init_code(factory: Address, owner: Address, salt: U256) -> Vec<u8> {
    use alloy::sol_types::SolCall;

    let call = SimpleAccountFactory::createAccountCall { owner, salt };
    let calldata = call.abi_encode();

    let mut init_code = Vec::new();
    init_code.extend_from_slice(factory.as_slice());
    init_code.extend_from_slice(&calldata);

    init_code
}

pub async fn fund_account(
    account_address: String,
    amount_wei: String,
    config: DeploymentConfig,
) -> Result<String, String> {
    // Parse account address
    let account_address = account_address
        .parse::<Address>()
        .map_err(|e| format!("Invalid account address: {e}"))?;
    let amount = amount_wei
        .parse::<U256>()
        .map_err(|e| format!("Invalid amount: {e}"))?;

    info!("Funding account {} with {} wei", account_address, amount);

    // Parse private key
    let signer = PrivateKeySigner::from_bytes(&FixedBytes::<32>::from_slice(
        &alloy::primitives::hex::decode(config.private_key.trim_start_matches("0x"))
            .map_err(|e| format!("Invalid private key: {e}"))?,
    ))
    .map_err(|e| format!("Failed to create signer: {e}"))?;

    let wallet = EthereumWallet::from(signer);

    // Set up provider
    let provider = ProviderBuilder::new().wallet(wallet).connect_http(
        config
            .rpc_url
            .parse()
            .map_err(|e| format!("Invalid RPC URL: {e}"))?,
    );

    // Get the account contract instance
    let account = SimpleAccount::new(account_address, &provider);

    // Send ETH to the account
    let tx = account
        .addDeposit()
        .value(amount)
        .send()
        .await
        .map_err(|e| format!("Failed to send funding transaction: {e}"))?;

    let receipt = tx
        .get_receipt()
        .await
        .map_err(|e| format!("Failed to get transaction receipt: {e}"))?;

    info!("Account funded in tx: {}", receipt.transaction_hash);

    // Check the deposit
    let deposit = account
        .getDeposit()
        .call()
        .await
        .map_err(|e| format!("Failed to get account deposit: {e}"))?;

    info!("Account deposit in EntryPoint: {} wei", deposit);

    Ok(format!("{:#x}", receipt.transaction_hash))
}
