use eyre::Result;
use sdk::{config::Config, utils::deployment_utils::deploy_contracts};
use std::{fs, path::PathBuf};
use url::Url;

pub async fn deploy_contracts_and_update_swift_config(
    node_url: Url,
    config_path: Option<PathBuf>,
) -> Result<()> {
    println!("Deploying contracts to node: {}", node_url);

    let contracts = deploy_contracts(node_url.clone()).await?;
    println!("Contracts deployed successfully:");
    println!("  AAFactory: {}", contracts.account_factory);
    println!("  WebAuthValidator: {}", contracts.passkey);
    println!("  SessionKeyValidator: {}", contracts.session);
    println!("  ExampleAuthServerPaymaster: {}", contracts.account_paymaster);

    let config_path =
        config_path.unwrap_or_else(Config::get_default_swift_config_path);
    println!("\nWriting config to path: {:?}", config_path);

    let config = Config::new(contracts, node_url);
    config.write_json(&config_path)?;

    println!("\nVerifying written config:");
    let written_json = fs::read_to_string(&config_path)?;
    println!("Written JSON content:\n{}", written_json);

    let written_config: Config = serde_json::from_str(&written_json)?;
    println!("\nParsed config verification:");
    println!("  Node URL: {}", written_config.node_url);
    println!("  AAFactory: {}", written_config.contracts.account_factory);
    println!("  WebAuthValidator: {}", written_config.contracts.passkey);
    println!("  SessionKeyValidator: {}", written_config.contracts.session);
    println!(
        "  ExampleAuthServerPaymaster: {}",
        written_config.contracts.account_paymaster
    );

    println!("\nSuccessfully updated and verified Swift config values");
    Ok(())
}
