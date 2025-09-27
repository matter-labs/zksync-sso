use alloy_web_core::erc4337::account::{DeploymentConfig, deploy_simple_account, fund_account};
use clap::Subcommand;
use eyre::Result;
use tracing::info;

#[derive(Subcommand)]
pub enum AccountCommands {
    /// Deploy a SimpleAccount
    Deploy {
        /// RPC URL for the network
        #[arg(long, default_value = "http://localhost:8545")]
        rpc_url: String,
        /// Private key for deployment
        #[arg(long, env = "PRIVATE_KEY")]
        private_key: Option<String>,
        /// Factory contract address
        #[arg(long)]
        factory: Option<String>,
        /// EntryPoint contract address
        #[arg(long)]
        entry_point: Option<String>,
        /// Owner address (defaults to signer address)
        #[arg(long)]
        owner: Option<String>,
        /// Salt for CREATE2 deployment
        #[arg(long, default_value = "0")]
        salt: u64,
    },
    /// Fund a SimpleAccount
    Fund {
        /// Account address to fund
        #[arg(long)]
        account: String,
        /// Amount to fund in ETH
        #[arg(long)]
        amount: f64,
        /// RPC URL for the network
        #[arg(long, default_value = "http://localhost:8545")]
        rpc_url: String,
        /// Private key for funding
        #[arg(long, env = "PRIVATE_KEY")]
        private_key: Option<String>,
    },
    /// Deploy with default local settings
    DeployLocal {
        /// Owner address (defaults to signer address)
        #[arg(long)]
        owner: Option<String>,
    },
}

pub async fn handle_command(cmd: AccountCommands) -> Result<()> {
    match cmd {
        AccountCommands::Deploy {
            rpc_url,
            private_key,
            factory,
            entry_point,
            owner,
            salt,
        } => {
            let config = DeploymentConfig {
                rpc_url,
                private_key: private_key.unwrap_or_else(|| {
                    // Default Anvil private key
                    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
                }),
                factory_address: factory,
                entry_point_address: entry_point,
                owner_address: owner,
                salt: Some(salt),
            };

            let deployment = deploy_simple_account(config)
                .await
                .map_err(|e| eyre::eyre!(e))?;

            info!("✅ SimpleAccount deployed successfully!");
            info!("  Factory: {}", deployment.factory_address);
            info!("  Account: {}", deployment.account_address);
            info!("  Owner: {}", deployment.owner);
            info!("  EntryPoint: {}", deployment.entry_point);
        }
        AccountCommands::Fund {
            account,
            amount,
            rpc_url,
            private_key,
        } => {
            let config = DeploymentConfig {
                rpc_url,
                private_key: private_key.unwrap_or_else(|| {
                    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
                }),
                ..Default::default()
            };

            // Convert ETH to wei
            let amount_wei = format!("{}", (amount * 1e18) as u128);

            let tx_hash = fund_account(account, amount_wei, config)
                .await
                .map_err(|e| eyre::eyre!(e))?;

            info!("✅ Account funded successfully!");
            info!("  Transaction: {}", tx_hash);
        }
        AccountCommands::DeployLocal { owner } => {
            // Use the local deployed addresses from our forge scripts
            let config = DeploymentConfig {
                rpc_url: "http://localhost:8545".to_string(),
                private_key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
                    .to_string(),
                factory_address: Some("0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9".to_string()),
                entry_point_address: Some("0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9".to_string()),
                owner_address: owner,
                salt: Some(0),
            };

            let deployment = deploy_simple_account(config)
                .await
                .map_err(|e| eyre::eyre!(e))?;

            info!("✅ SimpleAccount deployed successfully!");
            info!("  Factory: {}", deployment.factory_address);
            info!("  Account: {}", deployment.account_address);
            info!("  Owner: {}", deployment.owner);
            info!("  EntryPoint: {}", deployment.entry_point);
        }
    }

    Ok(())
}
