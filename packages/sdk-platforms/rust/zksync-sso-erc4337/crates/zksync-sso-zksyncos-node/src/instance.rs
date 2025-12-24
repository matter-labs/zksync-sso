pub mod anvil;
pub mod zksync_os_server;

use crate::{
    config::SpawnConfig,
    instance::{
        anvil::{DEFAULT_CHAIN_ID, spawn_l1_anvil},
        zksync_os_server::{
            build_repo, ensure_binary_exists, ensure_repo,
            spawn_server_process, wait_for_server,
        },
    },
};
use eyre::{Result, WrapErr};
use std::{fs, path::PathBuf, process::Child};
use url::Url;
pub use zksync_os_server::{
    DEFAULT_L1_PORT, DEFAULT_REPO_URL, DEFAULT_RPC_PORT,
};

pub struct ZkSyncOsInstance {
    rpc_url: Url,
    server_child: Option<Child>,
    anvil_child: Option<Child>,
    logs_dir: PathBuf,
}

impl std::fmt::Debug for ZkSyncOsInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZkSyncOsInstance")
            .field("rpc_url", &self.rpc_url)
            .field("logs_dir", &self.logs_dir)
            .finish()
    }
}

impl ZkSyncOsInstance {
    pub fn spawn() -> Result<Self> {
        Self::spawn_with_config(SpawnConfig::default())
    }

    pub fn spawn_with_config(config: SpawnConfig) -> Result<Self> {
        let SpawnConfig {
            repo_url,
            checkout_dir,
            l1_port,
            rpc_port,
            skip_build,
            print_logs,
        } = config;

        if skip_build {
            eyre::ensure!(
                checkout_dir.exists(),
                "zksync-os-server directory {} missing – run setup first or unset SSO_ZKSYNC_OS_SKIP_BUILD",
                checkout_dir.display()
            );
            ensure_binary_exists(&checkout_dir)?;
        } else {
            fs::create_dir_all(&checkout_dir).wrap_err(
                "failed to create zksync-os-server checkout directory",
            )?;
            ensure_repo(&checkout_dir, &repo_url)?;
            build_repo(&checkout_dir)?;
        }

        let logs_dir = checkout_dir.join("logs");
        fs::create_dir_all(&logs_dir)
            .wrap_err("failed to create logs directory for zksync-os-server")?;
        let anvil_child = spawn_l1_anvil(
            &checkout_dir,
            l1_port,
            DEFAULT_CHAIN_ID,
            &logs_dir,
            print_logs,
        )?;
        let server_child = spawn_server_process(
            &checkout_dir,
            rpc_port,
            l1_port,
            &logs_dir,
            print_logs,
        )?;
        let rpc_url = Url::parse(&format!("http://127.0.0.1:{rpc_port}"))
            .wrap_err("invalid RPC URL for zksync-os-server")?;
        wait_for_server(&rpc_url)?;

        Ok(Self {
            rpc_url,
            server_child: Some(server_child),
            anvil_child: Some(anvil_child),
            logs_dir,
        })
    }

    pub fn rpc_url(&self) -> &Url {
        &self.rpc_url
    }
}

impl Drop for ZkSyncOsInstance {
    fn drop(&mut self) {
        if let Some(child) = self.anvil_child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.anvil_child = None;
        if let Some(child) = self.server_child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.server_child = None;
    }
}
