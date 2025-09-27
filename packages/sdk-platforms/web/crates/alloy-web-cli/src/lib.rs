pub mod account;
pub mod anvil;
pub mod bundler;
pub mod infra;

pub mod test_utils;

use clap::{Parser, Subcommand};
use eyre::Result;

#[derive(Parser)]
#[command(name = "alloy-web-cli")]
#[command(about = "Infrastructure management for alloy-web")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Account management commands
    #[command(subcommand)]
    Account(account::AccountCommands),

    /// Bundler interaction commands
    #[command(subcommand)]
    Bundler(bundler::BundlerCommands),

    /// Infrastructure commands (Anvil, Alto)
    #[command(subcommand)]
    Infra(infra::InfraCommands),
}

pub async fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Account(cmd) => account::handle_command(cmd).await?,
        Commands::Bundler(cmd) => bundler::handle_command(cmd).await?,
        Commands::Infra(cmd) => infra::handle_command(cmd).await?,
    }

    Ok(())
}
