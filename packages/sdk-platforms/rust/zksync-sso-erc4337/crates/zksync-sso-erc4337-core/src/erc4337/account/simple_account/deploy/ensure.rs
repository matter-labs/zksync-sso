use crate::erc4337::{
    account::simple_account::contracts::SimpleAccountFactory,
    entry_point::EntryPoint,
};
use alloy::{primitives::Address, providers::Provider};
use eyre::WrapErr;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct DeployContractsResult {
    pub entry_point: Address,
    pub simple_account_factory: Address,
}

pub async fn ensure_entry_point_deployed<P>(
    provider: &P,
) -> eyre::Result<Address>
where
    P: Provider + Clone,
{
    if let Ok(s) = std::env::var("ZKSSO_ENTRYPOINT_ADDRESS")
        && let Ok(addr) = s.parse::<Address>()
    {
        let code =
            provider.get_code_at(addr).latest().await.unwrap_or_default();
        if !code.is_empty() {
            return Ok(addr);
        }
    }

    if let Ok(project_root) = std::env::var("ZKSSO_ERC4337_CONTRACTS_DIR") {
        let alto_path = PathBuf::from(project_root).join("alto.json");
        if let Ok(contents) = fs::read_to_string(alto_path)
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(&contents)
            && let Some(entry_s) = v.get("entrypoints").and_then(|x| x.as_str())
            && let Ok(addr) = entry_s.parse::<Address>()
        {
            let code =
                provider.get_code_at(addr).latest().await.unwrap_or_default();
            if !code.is_empty() {
                return Ok(addr);
            }
        }
    }

    let ep = EntryPoint::deploy(provider.clone())
        .await
        .wrap_err("deploy EntryPoint failed")?;
    Ok(*ep.address())
}

pub async fn ensure_simple_account_factory_deployed<P>(
    provider: &P,
    entry_point: Address,
) -> eyre::Result<Address>
where
    P: Provider + Clone,
{
    if let Ok(s) = std::env::var("ZKSSO_SIMPLE_ACCOUNT_FACTORY_ADDRESS")
        && let Ok(addr) = s.parse::<Address>()
    {
        let code =
            provider.get_code_at(addr).latest().await.unwrap_or_default();
        if !code.is_empty() {
            return Ok(addr);
        }
    }

    let factory = SimpleAccountFactory::deploy(provider.clone(), entry_point)
        .await
        .wrap_err("deploy SimpleAccountFactory failed")?;
    Ok(*factory.address())
}

pub async fn ensure_all_deployed<P>(
    provider: &P,
) -> eyre::Result<DeployContractsResult>
where
    P: Provider + Clone,
{
    let entry_point = ensure_entry_point_deployed(provider).await?;
    let factory =
        ensure_simple_account_factory_deployed(provider, entry_point).await?;
    Ok(DeployContractsResult { entry_point, simple_account_factory: factory })
}

#[cfg(test)]
mod tests {
    use crate::{
        chain::id::ChainId,
        erc4337::{
            account::simple_account::deploy::ensure::{
                ensure_all_deployed, ensure_entry_point_deployed,
                ensure_simple_account_factory_deployed,
            },
            client::test_utils::{
                AltoTestHelper, AltoTestHelperConfig, BundlerStatus,
            },
            entry_point::{
                config::EntryPointConfig, version::EntryPointVersion,
            },
        },
    };
    use alloy::{primitives::Address, providers::ProviderBuilder};
    use eyre::Result;

    #[tokio::test]
    #[ignore = "incomplete implementation"]
    async fn test_deploy_simple_account_flow() -> Result<()> {
        let provider = ProviderBuilder::new().connect_anvil_with_wallet();

        let alto_cfg = AltoTestHelperConfig {
            entrypoint: EntryPointConfig {
                address: "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108".parse()?,
                version: EntryPointVersion::V08,
                chain_id: ChainId::ETHEREUM_MAINNET,
            },
            executor_private_keys:
                "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
                    .to_string(),
            port: 4337,
            node_url: url::Url::parse("http://127.0.0.1:8545")?,
            safe_mode: false,
            utility_private_key:
                "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
                    .to_string(),
        };

        let mut alto = AltoTestHelper::new(provider.clone(), alto_cfg);
        alto.start().await?;
        assert!(matches!(alto.status().await?, BundlerStatus::Running));

        let entry = ensure_entry_point_deployed(&provider).await?;
        assert_ne!(entry, Address::ZERO);

        let factory =
            ensure_simple_account_factory_deployed(&provider, entry).await?;
        assert_ne!(factory, Address::ZERO);

        let result = ensure_all_deployed(&provider).await?;
        assert_eq!(result.entry_point, entry);
        assert_eq!(result.simple_account_factory, factory);

        alto.stop().await?;
        assert!(matches!(alto.status().await?, BundlerStatus::Stopped));
        Ok(())
    }
}
