use clap::{Parser, Subcommand};
use zksync_sso_erc4337_core::{
    chain::{Chain as CoreChain, id::ChainId as CoreChainId},
    config::{Config as CoreConfig, contracts::Contracts as CoreContracts},
    erc4337::entry_point::version::EntryPointVersion,
};

#[derive(Parser)]
#[command(name = "zksync-sso-erc4337")]
#[command(about = "A command-line interface for zkSync SSO ERC-4337")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    PrintConfig {
        #[arg(long)]
        rpc_url: String,

        #[arg(long)]
        bundler_url: String,

        #[arg(long)]
        entry_point_address: String,

        #[arg(long)]
        account_factory_address: String,

        #[arg(long)]
        webauthn_validator_address: String,

        #[arg(long)]
        eoa_validator_address: String,

        #[arg(long)]
        session_validator_address: String,

        #[arg(long)]
        guardian_executor_address: String,
    },
}

async fn handle_command(command: Commands) -> eyre::Result<()> {
    match command {
        Commands::PrintConfig {
            rpc_url,
            bundler_url,
            entry_point_address,
            account_factory_address,
            webauthn_validator_address,
            eoa_validator_address,
            session_validator_address,
            guardian_executor_address,
        } => {
            let config = CoreConfig::new(
                rpc_url.parse()?,
                bundler_url.parse()?,
                CoreChain::new(
                    CoreChainId::ETHEREUM_MAINNET,
                    EntryPointVersion::V08,
                    "Mainnet".to_string(),
                ),
                CoreContracts::from_string(
                    entry_point_address,
                    account_factory_address,
                    webauthn_validator_address,
                    eoa_validator_address,
                    session_validator_address.to_string(),
                    guardian_executor_address.to_string(),
                )?,
            );

            println!("Configuration: {config:?}");
        }
    }

    Ok(())
}

pub async fn run_cli() -> eyre::Result<()> {
    let cli = Cli::parse();

    let log_level =
        if cli.verbose { tracing::Level::DEBUG } else { tracing::Level::INFO };

    tracing_subscriber::fmt().with_max_level(log_level).init();

    handle_command(cli.command).await?;

    Ok(())
}
