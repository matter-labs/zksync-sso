pub mod anvil;
pub mod setup;
pub mod zksync_os_server;

use crate::{
    config::SpawnConfig,
    instance::{
        anvil::spawn_l1_anvil,
        setup::{ENTRYPOINT_ADDRESS, rpc_has_code, setup_zksync_os},
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
    DEFAULT_L1_RPC_URL, DEFAULT_L2_RPC_URL, DEFAULT_REPO_URL,
};
pub const DEFAULT_STATE_VERSION: &str = "v31.0";

pub struct ZkSyncOsInstance {
    rpc_url: Url,
    l1_rpc_url: Url,
    server_child: Option<Child>,
    anvil_child: Option<Child>,
    logs_dir: PathBuf,
}

impl std::fmt::Debug for ZkSyncOsInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZkSyncOsInstance")
            .field("rpc_url", &self.rpc_url)
            .field("l1_rpc_url", &self.l1_rpc_url)
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
            l1_rpc_url,
            l2_rpc_url,
            state_version,
            skip_build,
            print_logs,
            deploy_entrypoint,
            fund_wallet,
            deploy_test_contracts,
        } = config;

        if skip_build {
            eyre::ensure!(
                checkout_dir.exists(),
                "zksync-os-server directory {} missing – run setup first or ensure the binary is built",
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

        let db_dir = checkout_dir.join("db");
        if db_dir.exists() {
            let _ = fs::remove_dir_all(&db_dir);
        }

        let logs_dir = checkout_dir.join("logs");
        fs::create_dir_all(&logs_dir)
            .wrap_err("failed to create logs directory for zksync-os-server")?;
        let anvil_child = spawn_l1_anvil(
            &checkout_dir,
            &logs_dir,
            print_logs,
            &state_version,
        )?;
        let config_path = checkout_dir
            .join("local-chains")
            .join(&state_version)
            .join("default")
            .join("config.yaml");
        let server_child = spawn_server_process(
            &checkout_dir,
            &logs_dir,
            print_logs,
            &config_path,
        )?;
        wait_for_server(&l2_rpc_url)?;
        setup_zksync_os(
            &checkout_dir,
            &l2_rpc_url,
            print_logs,
            deploy_entrypoint,
            fund_wallet,
            deploy_test_contracts,
        )?;
        let entrypoint_ready = rpc_has_code(&l2_rpc_url, ENTRYPOINT_ADDRESS)?;
        eyre::ensure!(
            entrypoint_ready,
            "EntryPoint not deployed at {}",
            ENTRYPOINT_ADDRESS
        );

        Ok(Self {
            rpc_url: l2_rpc_url,
            l1_rpc_url,
            server_child: Some(server_child),
            anvil_child: Some(anvil_child),
            logs_dir,
        })
    }

    pub fn rpc_url(&self) -> &Url {
        &self.rpc_url
    }

    pub fn l1_rpc_url(&self) -> &Url {
        &self.l1_rpc_url
    }

    pub fn l2_rpc_url(&self) -> &Url {
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

        if let Some(checkout_dir) = self.logs_dir.parent() {
            let db_dir = checkout_dir.join("db");
            if db_dir.exists() {
                let _ = fs::remove_dir_all(&db_dir);
            }
        }
    }
}
