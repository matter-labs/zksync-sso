use alloy::providers::{Provider, ProviderBuilder};
use eyre::{Report, Result};
use serial_test::serial;
use std::{env, path::PathBuf};
use tokio::runtime::Runtime;
use zksync_sso_zksyncos_node::{
    config::SpawnConfig,
    instance::{
        DEFAULT_L1_PORT, DEFAULT_REPO_URL, DEFAULT_RPC_PORT, ZkSyncOsInstance,
    },
};

const CHECKOUT_ENV: &str = "SSO_ZKSYNC_OS_TEST_CHECKOUT";

fn ports_from_env() -> (u16, u16) {
    let l1_port = env::var("SSO_ZKSYNC_OS_TEST_L1_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_L1_PORT);
    let rpc_port = env::var("SSO_ZKSYNC_OS_TEST_RPC_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_RPC_PORT);
    (l1_port, rpc_port)
}

fn workspace_checkout_dir() -> PathBuf {
    env::var(CHECKOUT_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|_| SpawnConfig::default().checkout_dir)
}

#[test]
#[serial]
fn spawn_node_and_query_rpc() -> Result<()> {
    let repo_url = env::var("SSO_ZKSYNC_OS_TEST_REPO")
        .unwrap_or_else(|_| DEFAULT_REPO_URL.to_string());
    let (l1_port, rpc_port) = ports_from_env();
    let checkout_dir = workspace_checkout_dir();
    let server_bin =
        checkout_dir.join("target").join("release").join("zksync-os-server");
    eyre::ensure!(
        server_bin.exists(),
        "expected pre-built zksync-os-server binary at {}",
        server_bin.display()
    );

    let instance = ZkSyncOsInstance::spawn_with_config(SpawnConfig {
        repo_url,
        checkout_dir,
        l1_port,
        rpc_port,
        skip_build: true,
        print_logs: true,
    })?;

    let rpc_url = instance.rpc_url().clone();
    let rt = Runtime::new()?;
    rt.block_on(async move {
        let provider = ProviderBuilder::new().connect_http(rpc_url);
        let chain_id: u64 = provider.get_chain_id().await?;
        assert!(chain_id > 0, "chain id should be non-zero");
        Ok::<_, Report>(())
    })?;

    drop(instance);
    Ok(())
}
