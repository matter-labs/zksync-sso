pub mod commands;
pub mod deploy;
pub mod manager;

pub use commands::{InfraCommands, AllCommands, AnvilCommands};
pub use deploy::deploy_infrastructure;
pub use manager::InfrastructureManager;

use eyre::Result;

pub async fn handle_command(cmd: InfraCommands) -> Result<()> {
    let mut manager = InfrastructureManager::new();

    match cmd {
        InfraCommands::Deploy {
            rpc_url,
            private_key,
        } => deploy_infrastructure(rpc_url, private_key).await,

        InfraCommands::All(all_cmd) => match all_cmd {
            AllCommands::Start { fork_url, bundler } => manager.start_all(fork_url, &bundler).await,
            AllCommands::Stop => manager.stop_all().await,
        },

        InfraCommands::Anvil(anvil_cmd) => match anvil_cmd {
            AnvilCommands::Start { fork_url } => manager.start_anvil(fork_url).await,
            AnvilCommands::Stop => manager.stop_anvil().await,
        },

        InfraCommands::Bundler(bundler_cmd) => {
            // Delegate to bundler module
            crate::bundler::handle_command(bundler_cmd).await
        },

        InfraCommands::Status => manager.status().await,
    }
}