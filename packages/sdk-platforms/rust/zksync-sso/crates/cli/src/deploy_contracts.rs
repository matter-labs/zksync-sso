use eyre::Result;
use sdk::{
    api::utils::deploy_contracts,
    config::{Config, deploy_wallet::DeployWallet},
};
use std::{fs, path::PathBuf};
use url::Url;

pub async fn deploy_contracts_and_update_example_configs(
    node_url: Url,
    config_paths: Vec<PathBuf>,
) -> Result<()> {
    println!("Deploying contracts to node: {}", node_url);

    let contracts = deploy_contracts(node_url.clone()).await?;
    println!("Contracts deployed successfully:");
    println!("  AAFactory: {}", contracts.account_factory);
    println!("  WebAuthValidator: {}", contracts.passkey);
    println!("  SessionKeyValidator: {}", contracts.session);
    println!("  ExampleAuthServerPaymaster: {}", contracts.account_paymaster);
    println!("  Recovery: {}", contracts.recovery);

    let deploy_wallet = DeployWallet::random();
    let config =
        Config::new(contracts.clone(), node_url.clone(), deploy_wallet);

    for path in config_paths {
        let platform_name = path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("config"))
            .to_string_lossy();
        write_and_verify_config(&config, &path, &platform_name)?;
    }

    Ok(())
}

fn write_and_verify_config(
    config: &Config,
    config_path: &PathBuf,
    platform_name: &str,
) -> Result<()> {
    println!("\nWriting {} config to path: {:?}", platform_name, config_path);
    config.write_json(config_path)?;

    println!("\nVerifying written {} config:", platform_name);
    let written_json = fs::read_to_string(config_path)?;
    println!("Written {} JSON content:\n{}", platform_name, written_json);

    let written_config: Config = serde_json::from_str(&written_json)?;
    println!("\nParsed {} config verification:", platform_name);
    println!("  Node URL: {}", written_config.node_url);
    println!("  AAFactory: {}", written_config.contracts.account_factory);
    println!("  WebAuthValidator: {}", written_config.contracts.passkey);
    println!("  SessionKeyValidator: {}", written_config.contracts.session);
    println!(
        "  ExampleAuthServerPaymaster: {}",
        written_config.contracts.account_paymaster
    );
    println!("  Recovery: {}", written_config.contracts.recovery);
    println!(
        "\nSuccessfully updated and verified {} config values",
        platform_name
    );
    Ok(())
}
