use alloy_web_core::erc4337::infrastructure::{
    InfrastructureConfig, deploy_infrastructure as deploy_infra_core,
};
use eyre::{Result, eyre};
use tracing::{error, info};

pub async fn deploy_infrastructure(rpc_url: String, private_key: Option<String>) -> Result<()> {
    info!("Deploying ERC-4337 infrastructure to {}", rpc_url);

    let config = InfrastructureConfig {
        rpc_url,
        private_key: private_key.unwrap_or_else(|| {
            // Default Anvil private key
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
        }),
    };

    match deploy_infra_core(config).await {
        Ok(deployment) => {
            info!("✅ Infrastructure deployed successfully!");
            info!("EntryPoint: {}", deployment.entry_point);
            info!("SimpleAccountFactory: {}", deployment.factory);
            Ok(())
        }
        Err(e) => {
            error!("Failed to deploy infrastructure: {}", e);
            Err(eyre!("Deployment failed: {}", e))
        }
    }
}
