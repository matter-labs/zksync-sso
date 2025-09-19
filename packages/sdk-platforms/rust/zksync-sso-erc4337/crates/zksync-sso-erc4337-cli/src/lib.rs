use clap::{Parser, Subcommand};
use zksync_sso_erc4337_core::config::{
    Config as CoreConfig, contracts::Contracts as CoreContracts,
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
    Init {
        #[arg(long, default_value = "https://sepolia.era.zksync.dev")]
        rpc_url: String,

        #[arg(long, default_value = "https://bundler.example.com")]
        bundler_url: String,

        #[arg(
            long,
            default_value = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
        )]
        entry_point_address: String,

        #[arg(
            long,
            default_value = "0x9406Cc6185a346906296840746125a0E44976454"
        )]
        account_factory_address: String,
    },
    Config,
    Test,
}

async fn handle_command(command: Commands) -> eyre::Result<()> {
    match command {
        Commands::Init {
            rpc_url,
            bundler_url,
            entry_point_address,
            account_factory_address,
        } => {
            let config = CoreConfig::new(
                rpc_url,
                bundler_url,
                CoreContracts::from_string(
                    entry_point_address,
                    account_factory_address,
                )?,
            );

            println!("Configuration: {config:?}");
        }

        Commands::Config => {
            println!("Current configuration:");
            println!("  RPC URL: https://sepolia.era.zksync.dev");
            println!("  Bundler URL: https://bundler.example.com");
            println!(
                "  Entry Point: 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
            );
            println!(
                "  Account Factory: 0x9406Cc6185a346906296840746125a0E44976454"
            );
        }

        Commands::Test => {
            let config = CoreConfig::new(
                "https://sepolia.era.zksync.dev".to_string(),
                "https://bundler.example.com".to_string(),
                CoreContracts::from_string(
                    "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789".to_string(),
                    "0x9406Cc6185a346906296840746125a0E44976454".to_string(),
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
