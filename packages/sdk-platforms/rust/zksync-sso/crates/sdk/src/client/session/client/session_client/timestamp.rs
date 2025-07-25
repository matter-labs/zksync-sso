use crate::config::Config;
use alloy::{primitives::U64, providers::Provider};
use alloy_zksync::provider::zksync_provider;

pub async fn get_in_memory_node_timestamp(
    _config: Config,
) -> eyre::Result<U64> {
    let node_url = _config.node_url;
    let public_provider =
        zksync_provider().with_recommended_fillers().on_http(node_url);
    let method = "config_getCurrentTimestamp";
    let timestamp: u64 = public_provider.raw_request(method.into(), ()).await?;
    Ok(U64::from(timestamp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::spawn_node_and_deploy_contracts;

    #[tokio::test]
    async fn test_get_in_memory_node_timestamp() -> eyre::Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;

        // Act
        let timestamp = get_in_memory_node_timestamp(config.clone()).await?;

        // Assert
        eyre::ensure!(
            timestamp != U64::from(0),
            "Timestamp should not be zero"
        );

        // Clean
        drop(anvil_zksync);

        Ok(())
    }
}
