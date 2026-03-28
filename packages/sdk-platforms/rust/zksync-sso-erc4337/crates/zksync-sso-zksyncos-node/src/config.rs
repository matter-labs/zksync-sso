use crate::{
    env_helpers::env_bool,
    instance::{
        DEFAULT_L1_RPC_URL, DEFAULT_L2_RPC_URL, DEFAULT_REPO_URL,
        DEFAULT_STATE_VERSION,
    },
};
use std::{
    env,
    path::{Path, PathBuf},
};
use url::Url;

#[derive(Debug, Clone)]
pub struct SpawnConfig {
    pub repo_url: String,
    pub checkout_dir: PathBuf,
    pub l1_rpc_url: Url,
    pub l2_rpc_url: Url,
    pub state_version: String,
    pub skip_build: bool,
    pub print_logs: bool,
    pub deploy_entrypoint: bool,
    pub fund_wallet: bool,
    pub deploy_test_contracts: bool,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        let checkout_dir = workspace_checkout_guess();
        let repo_url = DEFAULT_REPO_URL.to_string();
        let l1_rpc_url =
            Url::parse(DEFAULT_L1_RPC_URL).expect("invalid DEFAULT_L1_RPC_URL");
        let l2_rpc_url =
            Url::parse(DEFAULT_L2_RPC_URL).expect("invalid DEFAULT_L2_RPC_URL");
        let state_version = DEFAULT_STATE_VERSION.to_string();

        let binary_path = checkout_dir
            .join("target")
            .join("release")
            .join("zksync-os-server");
        let skip_build = binary_path.exists();
        let print_logs = env_bool("SSO_ZKSYNC_OS_PRINT_LOGS").unwrap_or(true);
        let deploy_entrypoint =
            env_bool("SSO_ZKSYNC_OS_DEPLOY_ENTRYPOINT").unwrap_or(true);
        let fund_wallet = env_bool("SSO_ZKSYNC_OS_FUND_WALLET").unwrap_or(true);
        let deploy_test_contracts =
            env_bool("SSO_ZKSYNC_OS_DEPLOY_TEST_CONTRACTS").unwrap_or(true);

        Self {
            repo_url,
            checkout_dir,
            l1_rpc_url,
            l2_rpc_url,
            state_version,
            skip_build,
            print_logs,
            deploy_entrypoint,
            fund_wallet,
            deploy_test_contracts,
        }
    }
}

fn workspace_checkout_guess() -> PathBuf {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cwd_candidate = cwd.join("zksync-os-server");
    if cwd_candidate.exists() {
        return cwd_candidate;
    }

    let manifest_candidate = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("zksync-os-server"));

    if let Some(candidate) = manifest_candidate
        && candidate.exists()
    {
        return candidate;
    }

    cwd_candidate
}
