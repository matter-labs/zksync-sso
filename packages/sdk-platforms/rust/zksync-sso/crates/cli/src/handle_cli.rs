use crate::{
    config::ConfigLoader,
    handle_cli::commands::{
        Commands,
        Commands::{
            BuildAndDeployContracts, CreateSession, CreateSessionWithAccount,
            DeployAccount, DeployContracts, SendTransaction,
            SendTransactionWithAccount,
        },
        account::{
            deploy::deploy_account,
            session::{
                create_and_revoke::{
                    create_and_revoke_session,
                    create_and_revoke_session_with_account,
                },
                send::{send_transaction, send_transaction_with_account},
            },
        },
        deploy_contracts::{
            build_and_deploy_contracts_and_update_example_configs,
            deploy_contracts_and_update_example_configs,
        },
    },
};
use clap::Parser;

pub mod commands;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

pub async fn handle_cli() -> eyre::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        DeployContracts { node_url, config_paths } => {
            let paths_to_write = match config_paths {
                Some(paths) if !paths.is_empty() => paths,
                _ => ConfigLoader::get_all_default_config_paths(),
            };
            deploy_contracts_and_update_example_configs(
                node_url,
                paths_to_write,
            )
            .await?;
        }
        BuildAndDeployContracts { node_url, config_paths } => {
            build_and_deploy_contracts_and_update_example_configs(
                node_url,
                config_paths,
            )
            .await?;
        }
        DeployAccount => {
            deploy_account().await?;
        }
        CreateSession => {
            create_and_revoke_session().await?;
        }
        CreateSessionWithAccount { account_address } => {
            create_and_revoke_session_with_account(&account_address).await?;
        }
        SendTransaction => {
            send_transaction().await?;
        }
        SendTransactionWithAccount { account_address } => {
            send_transaction_with_account(&account_address).await?;
        }
    }

    Ok(())
}
