use crate::anvil::AnvilManager;
use crate::bundler::BundlerManager;
use eyre::Result;
use tracing::info;

pub struct InfrastructureManager {
    anvil_manager: AnvilManager,
    bundler_manager: BundlerManager,
}

impl Default for InfrastructureManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InfrastructureManager {
    pub fn new() -> Self {
        Self {
            anvil_manager: AnvilManager::new(),
            bundler_manager: BundlerManager::new().expect("Failed to create BundlerManager"),
        }
    }

    pub async fn start_anvil(&self, fork_url: Option<String>) -> Result<()> {
        self.anvil_manager.start(fork_url).await
    }

    pub async fn stop_anvil(&self) -> Result<()> {
        self.anvil_manager.stop().await
    }

    pub async fn start_all(&mut self, fork_url: Option<String>, bundler_type: &str) -> Result<()> {
        info!("Starting all infrastructure...");

        // Start Anvil
        self.start_anvil(fork_url).await?;

        // Deploy contracts
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        crate::infra::deploy::deploy_infrastructure("http://localhost:8545".to_string(), None).await?;

        // Start bundler using BundlerManager
        let bundler_type_enum = match bundler_type.to_lowercase().as_str() {
            "alto" => crate::bundler::BundlerType::Alto,
            "rundler" => crate::bundler::BundlerType::Rundler,
            _ => {
                return Err(eyre::eyre!(
                    "Unknown bundler type: {}. Use 'alto' or 'rundler'",
                    bundler_type
                ));
            }
        };

        self.bundler_manager
            .start_bundler(
                bundler_type_enum,
                "http://localhost:8545".to_string(),
                4337,
                "v0.7".to_string(),
            )
            .await?;

        info!("✅ All infrastructure started successfully");
        Ok(())
    }

    pub async fn stop_all(&mut self) -> Result<()> {
        info!("Stopping all infrastructure...");

        // Stop bundlers using BundlerManager
        self.bundler_manager.stop_all().await?;
        self.stop_anvil().await?;

        info!("✅ All infrastructure stopped");
        Ok(())
    }

    pub async fn status(&self) -> Result<()> {
        info!("Checking infrastructure status...");

        // Check Anvil using AnvilManager
        self.anvil_manager.status().await?;

        // Use BundlerManager for bundler status
        self.bundler_manager.status().await?;

        Ok(())
    }
}
