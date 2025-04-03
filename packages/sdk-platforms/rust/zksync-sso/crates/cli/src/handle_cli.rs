use clap::{Parser, Subcommand};
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

        /// Path to write the config file (defaults to swift/ZKsyncSSO/Sources/ZKsyncSSO/Config/config.json)
        #[arg(long)]
        config_path: Option<PathBuf>,
    },
}

pub async fn handle_cli() -> eyre::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::DeployContracts { node_url, config_path } => {
            super::deploy_contracts::deploy_contracts_and_update_swift_config(
                node_url,
                config_path,
            )
            .await?;
        }
    }

    Ok(())
}
