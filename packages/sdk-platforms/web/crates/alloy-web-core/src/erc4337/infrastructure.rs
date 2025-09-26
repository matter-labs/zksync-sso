use crate::erc4337::contracts::{EntryPoint, SimpleAccountFactory};
use alloy::network::EthereumWallet;
use alloy::primitives::{Address, FixedBytes};
use alloy::providers::{Provider, ProviderBuilder, WalletProvider};
use alloy::signers::local::PrivateKeySigner;
use tracing::{debug, info};

pub struct InfrastructureDeployment {
    pub entry_point: Address,
    pub factory: Address,
}

pub struct InfrastructureConfig {
    pub rpc_url: String,
    pub private_key: String,
}

impl Default for InfrastructureConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8545".to_string(),
            // Default Anvil private key
            private_key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
                .to_string(),
        }
    }
}

/// Deploy the ERC-4337 infrastructure using a generic provider
pub async fn deploy_infrastructure_impl<P>(provider: P) -> Result<InfrastructureDeployment, String>
where
    P: Provider + WalletProvider + Clone,
{
    info!("Deploying ERC-4337 infrastructure");

    // Deploy EntryPoint
    info!("Deploying EntryPoint...");

    // Prefer typed deploy helper if available
    let entry_point_instance = EntryPoint::deploy(provider.clone())
        .await
        .map_err(|e| format!("Failed to deploy EntryPoint: {e}"))?;

    let entry_point_address = entry_point_instance.address().to_owned();

    info!("EntryPoint deployed at: {}", entry_point_address);

    // Deploy SimpleAccountFactory
    info!("Deploying SimpleAccountFactory...");

    let simple_account_factory_instance =
        SimpleAccountFactory::deploy(provider, entry_point_address)
            .await
            .map_err(|e| format!("Failed to deploy SimpleAccountFactory: {e}"))?;

    let factory_address = simple_account_factory_instance.address().to_owned();

    info!("SimpleAccountFactory deployed at: {}", factory_address);

    // Log summary
    info!("=== Infrastructure Deployment Summary ===");
    info!("EntryPoint: {}", entry_point_address);
    info!("SimpleAccountFactory: {}", factory_address);
    info!("=========================================");

    Ok(InfrastructureDeployment {
        entry_point: entry_point_address,
        factory: factory_address,
    })
}

/// Deploy the ERC-4337 infrastructure (EntryPoint and SimpleAccountFactory)
pub async fn deploy_infrastructure(
    config: InfrastructureConfig,
) -> Result<InfrastructureDeployment, String> {
    // Parse private key
    let signer = PrivateKeySigner::from_bytes(&FixedBytes::<32>::from_slice(
        &alloy::primitives::hex::decode(config.private_key.trim_start_matches("0x"))
            .map_err(|e| format!("Invalid private key: {e}"))?,
    ))
    .map_err(|e| format!("Failed to create signer: {e}"))?;

    let wallet = EthereumWallet::from(signer);

    // Set up provider
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect(&config.rpc_url)
        .await
        .map_err(|e| format!("Failed to connect to provider: {e}"))?;

    deploy_infrastructure_impl(provider).await
}

/// Check if infrastructure is already deployed
pub async fn check_infrastructure(
    rpc_url: &str,
    entry_point: Option<Address>,
    factory: Option<Address>,
) -> Result<(bool, bool), String> {
    let provider = ProviderBuilder::new()
        .connect(rpc_url)
        .await
        .map_err(|e| format!("Failed to connect to provider: {e}"))?;

    let mut entry_point_exists = false;
    let mut factory_exists = false;

    // Check EntryPoint
    if let Some(addr) = entry_point {
        let code = provider
            .get_code_at(addr)
            .await
            .map_err(|e| format!("Failed to check EntryPoint code: {e}"))?;
        entry_point_exists = !code.is_empty();
        debug!("EntryPoint at {} exists: {}", addr, entry_point_exists);
    }

    // Check Factory
    if let Some(addr) = factory {
        let code = provider
            .get_code_at(addr)
            .await
            .map_err(|e| format!("Failed to check Factory code: {e}"))?;
        factory_exists = !code.is_empty();
        debug!("Factory at {} exists: {}", addr, factory_exists);
    }

    Ok((entry_point_exists, factory_exists))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eyre::Ok;

    #[tokio::test]
    async fn test_deploy_infrastructure_with_anvil() -> eyre::Result<()> {
        // Create provider with in-memory Anvil node and wallet
        let provider = ProviderBuilder::new().connect_anvil_with_wallet();

        // Deploy infrastructure on the same provider
        let result = deploy_infrastructure_impl(provider.clone()).await;

        // Verify deployment succeeded
        assert!(result.is_ok(), "Deployment failed: {:?}", result.err());

        let deployment = result.unwrap();

        // Verify addresses are not zero
        assert_ne!(deployment.entry_point, Address::ZERO);
        assert_ne!(deployment.factory, Address::ZERO);

        // Verify contracts are deployed by checking code using the same provider

        let entry_point_code = provider
            .get_code_at(deployment.entry_point)
            .await
            .expect("Failed to get EntryPoint code");
        assert!(!entry_point_code.is_empty(), "EntryPoint code is empty");

        let factory_code = provider
            .get_code_at(deployment.factory)
            .await
            .expect("Failed to get Factory code");
        assert!(!factory_code.is_empty(), "Factory code is empty");

        Ok(())
    }
}
