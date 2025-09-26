use clap::Subcommand;
use eyre::Result;

use super::manager::{BundlerManager, BundlerType};

#[derive(Subcommand)]
pub enum BundlerCommands {
    /// Start the default bundler (Rundler)
    Start {
        /// RPC URL for the bundler
        #[arg(long, default_value = "http://localhost:8545")]
        rpc_url: String,
        /// Port for the bundler RPC
        #[arg(long, default_value = "4337")]
        port: u16,
        /// EntryPoint version
        #[arg(long, default_value = "v0.7")]
        entry_point: String,
    },
    /// Manage Alto bundler
    #[command(subcommand)]
    Alto(BundlerServiceCommands),
    /// Manage Rundler bundler
    #[command(subcommand)]
    Rundler(BundlerServiceCommands),
    /// Stop bundler services
    Stop {
        /// Specific bundler to stop (alto, rundler, or all)
        #[arg(long)]
        bundler: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum BundlerServiceCommands {
    /// Start the bundler service
    Start {
        /// RPC URL for the bundler
        #[arg(long, default_value = "http://localhost:8545")]
        rpc_url: String,
        /// Port for the bundler RPC
        #[arg(long, default_value = "4337")]
        port: u16,
        /// EntryPoint version
        #[arg(long, default_value = "v0.7")]
        entry_point: String,
    },
    /// Stop the bundler service
    Stop,
}

pub async fn handle_command(command: BundlerCommands) -> Result<()> {
    let mut manager = BundlerManager::new()?;

    match command {
        BundlerCommands::Start {
            rpc_url,
            port,
            entry_point,
        } => {
            // Default to Rundler
            manager
                .start_bundler(BundlerType::Rundler, rpc_url, port, entry_point)
                .await
        }
        BundlerCommands::Alto(command) => match command {
            BundlerServiceCommands::Start {
                rpc_url,
                port,
                entry_point,
            } => {
                manager
                    .start_bundler(BundlerType::Alto, rpc_url, port, entry_point)
                    .await
            }
            BundlerServiceCommands::Stop => {
                manager.stop("alto").await
            }
        },
        BundlerCommands::Rundler(command) => match command {
            BundlerServiceCommands::Start {
                rpc_url,
                port,
                entry_point,
            } => {
                manager
                    .start_bundler(BundlerType::Rundler, rpc_url, port, entry_point)
                    .await
            }
            BundlerServiceCommands::Stop => {
                manager.stop("rundler").await
            }
        },
        BundlerCommands::Stop { bundler } => {
            if let Some(bundler_name) = bundler {
                manager.stop(&bundler_name).await
            } else {
                manager.stop_all().await
            }
        }
    }
}
