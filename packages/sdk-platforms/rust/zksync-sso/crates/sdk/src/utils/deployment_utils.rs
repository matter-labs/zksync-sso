use crate::{
    config::contracts::SSOContracts,
    utils::contract_deployed::check_contracts_deployed,
};
use rand::RngCore;
use std::{env, fs, path::PathBuf, process::Command};

pub async fn deploy_contracts(
    node_url: url::Url,
) -> eyre::Result<SSOContracts> {
    println!("Node URL: {node_url}");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    println!("Manifest directory: {manifest_dir:?}");

    let contracts_dir = PathBuf::from(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("packages/contracts");
    println!("Contracts directory: {contracts_dir:?}");
    println!("Contracts directory exists: {}", contracts_dir.exists());
    println!(
        "Contracts directory is absolute: {}",
        contracts_dir.is_absolute()
    );

    let mut random_bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut random_bytes);
    let random_suffix = alloy::hex::encode(random_bytes);
    let config_filename = format!("hardhat.config.{random_suffix}.ts");
    let config_path = contracts_dir.join(&config_filename);
    println!("Config path: {config_path:?}");

    let config_content = format!(
        r#"import "@typechain/hardhat";
import "@matterlabs/hardhat-zksync";
import "@nomicfoundation/hardhat-chai-matchers";
import "./scripts/deploy";
import "./scripts/publish";
import "./scripts/upgrade";

import {{ HardhatUserConfig }} from "hardhat/config";

const config: HardhatUserConfig = {{
  paths: {{
    sources: "src",
    deployPaths: "scripts",
  }},
  defaultNetwork: "inMemoryNode",
  networks: {{
    inMemoryNode: {{
      url: "{node_url}",
      ethNetwork: "localhost", // in-memory node doesn't support eth node; removing this line will cause an error
      zksync: true,
    }},
  }},
  zksolc: {{
    version: "1.5.12",
    settings: {{
      enableEraVMExtensions: true,
    }},
  }},
  solidity: {{
    version: "0.8.28",
    settings: {{
      evmVersion: "cancun",
      codegen: "yul",
    }}
  }},
}};

export default config;"#
    );

    println!("Writing config to {config_path:?}");
    fs::write(&config_path, &config_content)?;
    println!("Config file exists: {}", config_path.exists());
    println!("Config file contents: {}", fs::read_to_string(&config_path)?);

    println!("Running pnpm deploy from {contracts_dir:?}");
    let output = Command::new("pnpm")
        .current_dir(&contracts_dir)
        .arg("run")
        .arg("deploy")
        .arg("--network")
        .arg("inMemoryNode")
        .arg("--config")
        .arg(&config_filename)
        .output()?;

    let cleanup_result = fs::remove_file(&config_path);
    println!("Config file cleanup result: {cleanup_result:?}");
    println!("Config file still exists: {}", config_path.exists());

    println!("Command output: {output:?}");
    println!("Command stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Command stderr: {}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        return Err(eyre::eyre!(
            "Failed to deploy contracts: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();

    let account_factory = extract_contract_address(&lines, "AAFactory")?;
    let passkey = extract_contract_address(&lines, "WebAuthValidator")?;
    let session = extract_contract_address(&lines, "SessionKeyValidator")?;
    let account_paymaster =
        extract_contract_address(&lines, "ExampleAuthServerPaymaster")?;
    let recovery =
        extract_contract_address(&lines, "GuardianRecoveryValidator")?;

    let contracts = SSOContracts::with_address_strs(
        account_factory,
        passkey,
        session,
        account_paymaster,
        recovery,
    )?;

    check_contracts_deployed(&node_url, &contracts).await?;

    println!("Contracts deployed: {contracts:?}");

    Ok(contracts)
}

fn extract_contract_address<'a>(
    lines: &'a [&'a str],
    contract_name: &str,
) -> eyre::Result<&'a str> {
    lines
        .iter()
        .find(|line| {
            line.contains(&format!(
                "{contract_name} proxy contract deployed at:"
            )) || line
                .contains(&format!("{contract_name} contract deployed at:"))
        })
        .and_then(|line| line.split(": ").nth(1))
        .map(|addr| addr.trim())
        .ok_or_else(|| eyre::eyre!("Failed to find {} address", contract_name))
}
