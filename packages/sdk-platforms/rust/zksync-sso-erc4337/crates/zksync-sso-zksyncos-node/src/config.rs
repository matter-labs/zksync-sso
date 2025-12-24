use crate::{
    env_helpers::env_bool,
    instance::{DEFAULT_L1_PORT, DEFAULT_REPO_URL, DEFAULT_RPC_PORT},
};
use std::{
    env,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct SpawnConfig {
    pub repo_url: String,
    pub checkout_dir: PathBuf,
    pub l1_port: u16,
    pub rpc_port: u16,
    pub skip_build: bool,
    pub print_logs: bool,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        let checkout_dir = env::var("SSO_ZKSYNC_OS_SERVER_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_checkout_guess());

        let repo_url = env::var("SSO_ZKSYNC_OS_SERVER_REPO")
            .unwrap_or_else(|_| DEFAULT_REPO_URL.to_string());

        let l1_port = env::var("SSO_ZKSYNC_OS_L1_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_L1_PORT);

        let rpc_port = env::var("SSO_ZKSYNC_OS_RPC_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_RPC_PORT);

        let skip_build = env_bool("SSO_ZKSYNC_OS_SKIP_BUILD").unwrap_or(true);
        let print_logs = env_bool("SSO_ZKSYNC_OS_PRINT_LOGS").unwrap_or(true);

        Self {
            repo_url,
            checkout_dir,
            l1_port,
            rpc_port,
            skip_build,
            print_logs,
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
