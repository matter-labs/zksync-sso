use crate::config::ConfigLoader;
use eyre::Result;
use sdk::{
    api::utils::deploy_contracts,
    config::{Config, deploy_wallet::DeployWallet},
};
use std::{fs, path::PathBuf, process::Command};
use url::Url;

pub async fn deploy_contracts_and_update_example_configs(
    node_url: Url,
    config_paths: Vec<PathBuf>,
) -> Result<()> {
    println!("Deploying contracts to node: {node_url}");

    let contracts = deploy_contracts(node_url.clone()).await?;
    println!("Contracts deployed successfully:");
    println!("  AAFactory: {}", contracts.account_factory);
    println!("  WebAuthValidator: {}", contracts.passkey);
    println!("  SessionKeyValidator: {}", contracts.session);
    println!("  ExampleAuthServerPaymaster: {}", contracts.account_paymaster);
    println!("  Recovery: {}", contracts.recovery);

    let deploy_wallet = Some(DeployWallet::rich_wallet());
    let config = Config::new(contracts, node_url.clone(), deploy_wallet);

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
    println!("\nWriting {platform_name} config to path: {config_path:?}");
    config.write_json(config_path)?;

    println!("\nVerifying written {platform_name} config:");
    let written_json = fs::read_to_string(config_path)?;
    println!("Written {platform_name} JSON content:\n{written_json}");

    let written_config: Config = serde_json::from_str(&written_json)?;
    println!("\nParsed {platform_name} config verification:");
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
        "\nSuccessfully updated and verified {platform_name} config values"
    );
    Ok(())
}

pub async fn build_and_deploy_contracts_and_update_example_configs(
    node_url: Url,
    config_paths: Option<Vec<PathBuf>>,
) -> Result<()> {
    println!("Building contracts...");

    let contracts_dir = std::env::current_dir()?
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.join("packages").join("contracts"))
        .ok_or_else(|| eyre::eyre!("Could not find contracts directory"))?;

    let output = Command::new("pnpm")
        .arg("build")
        .current_dir(&contracts_dir)
        .output()?;

    if !output.status.success() {
        return Err(eyre::eyre!(
            "Failed to build contracts: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    println!("Contracts built successfully!");

    let paths_to_write = match config_paths {
        Some(paths) if !paths.is_empty() => paths,
        _ => ConfigLoader::get_all_default_config_paths(),
    };

    deploy_contracts_and_update_example_configs(node_url, paths_to_write).await
}
