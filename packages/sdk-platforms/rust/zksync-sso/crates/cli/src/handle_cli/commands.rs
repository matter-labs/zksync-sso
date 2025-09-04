use clap::Subcommand;
use std::path::PathBuf;
use url::Url;

pub mod account;
pub mod deploy_contracts;

#[derive(Subcommand)]
pub enum Commands {
    /// Deploy contracts and update Swift config values
    DeployContracts {
        /// The URL of the zkSync node (defaults to http://localhost:8011/)
        #[arg(long, default_value = "http://localhost:8011/")]
        node_url: Url,

        /// Paths to write the config files to. If not provided or if empty, defaults to Swift and React Native example paths.
        #[arg(long)]
        config_paths: Option<Vec<PathBuf>>,
    },
    /// Build contracts and then deploy them with config updates
    BuildAndDeployContracts {
        /// The URL of the zkSync node (defaults to http://localhost:8011/)
        #[arg(long, default_value = "http://localhost:8011/")]
        node_url: Url,

        /// Paths to write the config files to. If not provided or if empty, defaults to Swift and React Native example paths.
        #[arg(long)]
        config_paths: Option<Vec<PathBuf>>,
    },
    /// Deploy a modular account with hardcoded test configuration
    DeployAccount,
    /// Create a session for the deterministically deployed account using hardcoded test configuration
    CreateSession,
    /// Create a session for a specific account address using hardcoded test configuration
    CreateSessionWithAccount {
        /// The address of the deployed account to create the session for
        #[arg(long)]
        account_address: String,
    },
    /// Send a transaction using session for the deterministically deployed account using hardcoded test configuration
    SendTransaction,
    /// Send a transaction using session for a specific account address using hardcoded test configuration
    SendTransactionWithAccount {
        /// The address of the deployed account to send the transaction from
        #[arg(long)]
        account_address: String,
    },
}
