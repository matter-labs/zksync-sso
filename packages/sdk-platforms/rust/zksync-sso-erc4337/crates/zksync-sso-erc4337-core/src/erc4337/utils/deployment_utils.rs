use crate::{
    config::contracts::Contracts,
    erc4337::utils::check_deployed::{
        Contract, check_contract_deployed, check_contracts_deployed,
    },
};
use alloy::{
    primitives::{Address, address},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};
use std::{env, path::PathBuf, process::Command, str::FromStr};

fn find_contracts_directory(manifest_dir: &str) -> eyre::Result<PathBuf> {
    let mut search_dir = PathBuf::from(manifest_dir);

    loop {
        let contracts_dir = search_dir.join("erc4337-contracts");
        if contracts_dir.exists() && contracts_dir.is_dir() {
            return Ok(contracts_dir);
        }

        let packages_contracts_dir =
            search_dir.join("packages").join("erc4337-contracts");
        if packages_contracts_dir.exists() && packages_contracts_dir.is_dir() {
            return Ok(packages_contracts_dir);
        }

        if let Some(parent) = search_dir.parent() {
            search_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    Err(eyre::eyre!("Could not find erc4337-contracts directory"))
}

fn get_manifest_dir() -> eyre::Result<String> {
    env::var("CARGO_MANIFEST_DIR").or_else(|_| -> eyre::Result<String> {
        let current_dir = std::env::current_dir()?;
        let mut search_dir = current_dir.clone();

        loop {
            let cargo_toml = search_dir.join("Cargo.toml");
            if cargo_toml.exists()
                && let Ok(content) = std::fs::read_to_string(&cargo_toml)
                && content.contains("[workspace]")
            {
                return Ok(search_dir.to_string_lossy().to_string());
            }

            if let Some(parent) = search_dir.parent() {
                search_dir = parent.to_path_buf();
            } else {
                break;
            }
        }

        Ok(current_dir.to_string_lossy().to_string())
    })
}

pub async fn deploy_contracts_default(
    node_url: &url::Url,
) -> eyre::Result<Contracts> {
    let entrypoint_address =
        address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");
    deploy_contracts(node_url, entrypoint_address).await
}

pub async fn deploy_contracts(
    node_url: &url::Url,
    entry_point: Address,
) -> eyre::Result<Contracts> {
    println!("Deploying contracts to {}", node_url);

    let manifest_dir = get_manifest_dir()?;
    println!("Manifest directory: {manifest_dir:?}");

    let contracts_dir = find_contracts_directory(&manifest_dir)?;
    println!("Contracts directory: {contracts_dir:?}");
    println!("Contracts directory exists: {}", contracts_dir.exists());
    println!(
        "Contracts directory is absolute: {}",
        contracts_dir.is_absolute()
    );

    println!("Running pnpm deploy from {contracts_dir:?}");

    let provider = {
        let signer_private_key = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
        let signer = PrivateKeySigner::from_str(signer_private_key)?;
        let ethereum_wallet = alloy::network::EthereumWallet::new(signer);

        ProviderBuilder::new()
            .wallet(ethereum_wallet)
            .connect_http(node_url.clone())
    };

    check_contract_deployed(
        &Contract { address: entry_point, name: "EntryPoint".to_string() },
        provider.clone(),
    )
    .await?;

    let rpc_url_string = node_url.to_string();
    let output = Command::new("forge")
        .current_dir(&contracts_dir)
        .arg("script")
        .arg("script/Deploy.s.sol")
        .arg("--rpc-url")
        .arg(rpc_url_string)
        .arg("--broadcast")
        .arg("--private-key")
        .arg("0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6")
        .arg("--sig")
        .arg("deployAll()")
        .output()?;

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

    let eoa_validator = extract_contract_address(&lines, "EOAKeyValidator")?;
    let session_validator =
        extract_contract_address(&lines, "SessionKeyValidator")?;
    let webauthn_validator =
        extract_contract_address(&lines, "WebAuthnValidator")?;
    let guardian_executor =
        extract_contract_address(&lines, "GuardianExecutor")?;
    let account_factory = extract_contract_address(&lines, "MSAFactory")?;

    let contracts = Contracts {
        eoa_validator,
        session_validator,
        webauthn_validator,
        account_factory,
        entry_point,
        guardian_executor,
    };

    check_contracts_deployed(&contracts, provider).await?;

    println!("Contracts deployed: {contracts:?}");

    Ok(contracts)
}

fn extract_contract_address<'a>(
    lines: &'a [&'a str],
    contract_name: &str,
) -> eyre::Result<Address> {
    lines
        .iter()
        .find(|line| line.contains(&format!("{contract_name}: ")))
        .and_then(|line| line.split(": ").nth(1))
        .map(|addr| addr.trim())
        .ok_or_else(|| eyre::eyre!("Failed to find {} address", contract_name))
        .and_then(|addr| {
            Address::from_str(addr)
                .map_err(|_| eyre::eyre!("Failed to parse address"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{node_bindings::Anvil, primitives::address};
    use std::env;
    use url::Url;

    #[test]
    fn test_get_manifest_dir_with_env_var() -> eyre::Result<()> {
        let test_path = "/test/manifest/dir";
        unsafe {
            env::set_var("CARGO_MANIFEST_DIR", test_path);
        }

        let result = get_manifest_dir()?;
        assert_eq!(result, test_path);

        unsafe {
            env::remove_var("CARGO_MANIFEST_DIR");
        }
        Ok(())
    }

    #[test]
    fn test_get_manifest_dir_fallback() -> eyre::Result<()> {
        unsafe {
            env::remove_var("CARGO_MANIFEST_DIR");
        }

        let result = get_manifest_dir()?;

        assert!(!result.is_empty());
        assert!(PathBuf::from(&result).exists());

        println!("Fallback manifest dir: {}", result);
        Ok(())
    }

    #[test]
    fn test_find_contracts_directory() -> eyre::Result<()> {
        let current_dir = std::env::current_dir()?;
        let contracts_dir =
            find_contracts_directory(&current_dir.to_string_lossy())?;

        println!(
            "Found contracts directory: {}",
            contracts_dir.to_string_lossy()
        );
        assert!(contracts_dir.exists());
        assert!(contracts_dir.is_dir());
        assert!(contracts_dir.ends_with("erc4337-contracts"));

        Ok(())
    }

    #[tokio::test]
    #[ignore = "manual test"]
    async fn test_deploy_contracts() -> eyre::Result<()> {
        let anvil = Anvil::new().spawn();
        let node_url: Url = anvil.endpoint().parse()?;

        let entrypoint_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let contracts = deploy_contracts(&node_url, entrypoint_address).await?;

        println!("Contracts deployed: {contracts:?}");

        Ok(())
    }
}
