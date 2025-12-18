use alloy::providers::{Provider, ProviderBuilder};
use eyre::Result;
use std::{env, path::PathBuf};
use zksync_sso_zksyncos_node::{
    config::SpawnConfig,
    instance::{DEFAULT_REPO_URL, ZkSyncOsInstance},
};

const CHECKOUT_ENV: &str = "SSO_ZKSYNC_OS_TEST_CHECKOUT";

fn workspace_checkout_dir() -> PathBuf {
    env::var(CHECKOUT_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|_| SpawnConfig::default().checkout_dir)
}

#[tokio::test]
#[ignore = "manual test only"]
async fn spawn_node_and_query_l1_rpc() -> Result<()> {
    let repo_url = env::var("SSO_ZKSYNC_OS_TEST_REPO")
        .unwrap_or_else(|_| DEFAULT_REPO_URL.to_string());
    let checkout_dir = workspace_checkout_dir();
    let config =
        SpawnConfig { repo_url, checkout_dir, ..SpawnConfig::default() };
    let server_bin = config
        .checkout_dir
        .join("target")
        .join("release")
        .join("zksync-os-server");
    eyre::ensure!(
        server_bin.exists(),
        "expected pre-built zksync-os-server binary at {}",
        server_bin.display()
    );

    let instance = ZkSyncOsInstance::spawn_with_config(SpawnConfig {
        skip_build: true,
        print_logs: true,
        deploy_entrypoint: true,
        fund_wallet: true,
        deploy_test_contracts: true,
        ..config
    })?;

    let l1_provider =
        ProviderBuilder::new().connect_http(instance.l1_rpc_url().clone());
    let l1_chain_id: u64 = l1_provider.get_chain_id().await?;
    assert!(l1_chain_id > 0, "l1 chain id should be non-zero");

    drop(instance);
    Ok(())
}

#[tokio::test]
#[ignore = "manual test only"]
async fn spawn_node_and_query_zksync_os_rpc() -> Result<()> {
    let repo_url = env::var("SSO_ZKSYNC_OS_TEST_REPO")
        .unwrap_or_else(|_| DEFAULT_REPO_URL.to_string());
    let checkout_dir = workspace_checkout_dir();
    let config =
        SpawnConfig { repo_url, checkout_dir, ..SpawnConfig::default() };
    let server_bin = config
        .checkout_dir
        .join("target")
        .join("release")
        .join("zksync-os-server");
    eyre::ensure!(
        server_bin.exists(),
        "expected pre-built zksync-os-server binary at {}",
        server_bin.display()
    );

    let instance = ZkSyncOsInstance::spawn_with_config(SpawnConfig {
        skip_build: true,
        print_logs: true,
        deploy_entrypoint: true,
        fund_wallet: true,
        deploy_test_contracts: true,
        ..config
    })?;

    let provider =
        ProviderBuilder::new().connect_http(instance.l2_rpc_url().clone());
    let chain_id: u64 = provider.get_chain_id().await?;
    assert!(chain_id > 0, "chain id should be non-zero");

    drop(instance);
    Ok(())
}
