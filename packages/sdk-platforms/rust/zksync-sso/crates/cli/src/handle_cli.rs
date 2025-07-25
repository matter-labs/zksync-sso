use crate::handle_cli::Commands::{BuildAndDeployContracts, DeployContracts};
use clap::{Parser, Subcommand};
use sdk::config::Config;
use std::path::PathBuf;
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
}

pub async fn handle_cli() -> eyre::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        DeployContracts { node_url, config_paths } => {
            let paths_to_write = match config_paths {
                Some(paths) if !paths.is_empty() => paths,
                _ => vec![
                    Config::get_default_swift_config_path(),
                    Config::get_default_react_native_config_path(),
                ],
            };
            super::deploy_contracts::deploy_contracts_and_update_example_configs(
                node_url,
                paths_to_write,
            )
            .await?;
        }
        BuildAndDeployContracts { node_url, config_paths } => {
            super::deploy_contracts::build_and_deploy_contracts_and_update_example_configs(
                node_url,
                config_paths,
            )
            .await?;
        }
    }

    Ok(())
}
