use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{error, info};

use super::alto::AltoBundler;
use super::rundler::RundlerBundler;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BundlerType {
    Alto,
    Rundler,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BundlerProcess {
    pub pid: u32,
    pub name: String,
    pub port: u16,
    pub rpc_url: String,
    pub started_at: u64,
    pub config: BundlerConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BundlerConfig {
    pub rpc_url: String,
    pub port: u16,
    pub entry_point: String,
    pub chain_id: u64,
    pub unsafe_mode: bool,
    pub fork_url: Option<String>,
    pub fork_block: Option<u64>,
    pub auto_mine: bool,
    pub use_docker: Option<bool>,
}

impl Default for BundlerConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8545".to_string(),
            port: 4337,
            entry_point: "v0.7".to_string(),
            chain_id: 11155111, // Sepolia
            unsafe_mode: true,
            fork_url: None,
            fork_block: None,
            auto_mine: true,
            use_docker: None,
        }
    }
}

pub struct BundlerManager {
    processes: HashMap<String, Child>,
    state_file: PathBuf,
    alto_bundler: AltoBundler,
    rundler_bundler: RundlerBundler,
}

impl BundlerManager {
    pub fn new() -> Result<Self> {
        let state_dir = dirs::data_dir()
            .ok_or_else(|| eyre::eyre!("Could not find data directory"))?
            .join("alloy-web");

        fs::create_dir_all(&state_dir)?;

        let state_file = state_dir.join("bundler_processes.json");

        Ok(Self {
            processes: HashMap::new(),
            state_file,
            alto_bundler: AltoBundler::new(),
            rundler_bundler: RundlerBundler::new(),
        })
    }

    pub async fn start_bundler(
        &mut self,
        bundler_type: BundlerType,
        rpc_url: String,
        port: u16,
        entry_point: String,
    ) -> Result<()> {
        let config = BundlerConfig {
            rpc_url,
            port,
            entry_point,
            chain_id: 31337, // Local chain ID
            unsafe_mode: true,
            fork_url: None,
            fork_block: None,
            auto_mine: true,
            use_docker: Some(bundler_type == BundlerType::Rundler), // Use Docker for Rundler by default
        };

        match bundler_type {
            BundlerType::Alto => self.alto_bundler.start(&config, &self.state_file).await,
            BundlerType::Rundler => self.rundler_bundler.start(&config, &self.state_file).await,
        }
    }

    pub async fn start_anvil(&mut self, config: BundlerConfig) -> Result<()> {
        let name = "anvil".to_string();

        // Check if already running
        if self.is_running(&name).await? {
            info!("Anvil is already running");
            return Ok(());
        }

        info!("Starting Anvil on port {}", config.port);

        let mut cmd = Command::new("anvil");
        cmd.arg("--port").arg(config.port.to_string());

        if let Some(url) = &config.fork_url {
            info!("Forking from {}", url);
            cmd.arg("--fork-url").arg(url);

            if let Some(block) = config.fork_block {
                cmd.arg("--fork-block-number").arg(block.to_string());
            }
        }

        if !config.auto_mine {
            cmd.arg("--no-mining");
        }

        // Add common test accounts
        cmd.arg("--accounts").arg("10");
        cmd.arg("--balance").arg("10000");
        cmd.arg("--mnemonic")
            .arg("test test test test test test test test test test test junk");

        // Run in silent mode to reduce output
        cmd.arg("--silent");

        // Start the process in the background
        let child = cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;

        let pid = child.id();

        // Don't store the child process as it will be detached
        // self.processes.insert(name.clone(), child);

        // Save state
        let process_info = BundlerProcess {
            pid,
            name: name.clone(),
            port: config.port,
            rpc_url: format!("http://localhost:{}", config.port),
            started_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            config: config.clone(),
        };

        self.save_state(&process_info)?;

        // Wait a moment for the process to start
        sleep(Duration::from_secs(3)).await;

        // Verify it's running
        if self.is_running(&name).await? {
            info!("✅ Anvil started successfully on port {}", config.port);
            info!("RPC URL: http://localhost:{}", config.port);
            info!("Process ID: {}", pid);
        } else {
            error!("Failed to start Anvil");
            return Err(eyre::eyre!("Anvil failed to start"));
        }

        Ok(())
    }

    pub async fn start_all(&mut self, config: BundlerConfig) -> Result<()> {
        info!("Starting all bundler services...");

        // Start Anvil first
        let mut anvil_config = config.clone();
        anvil_config.port = 8545;
        self.start_anvil(anvil_config).await?;

        // Wait for Anvil to be ready
        sleep(Duration::from_secs(3)).await;

        // Start Rundler
        let mut rundler_config = config.clone();
        rundler_config.port = 4337;
        rundler_config.rpc_url = "http://localhost:8545".to_string();
        self.rundler_bundler.start(&rundler_config, &self.state_file).await?;

        info!("✅ All bundler services started successfully!");
        info!("Anvil RPC: http://localhost:8545");
        info!("Rundler Bundler RPC: http://localhost:4337");

        Ok(())
    }

    pub async fn start_all_with_alto(&mut self, config: BundlerConfig) -> Result<()> {
        info!("Starting all bundler services with Alto...");

        // Start Anvil first
        let mut anvil_config = config.clone();
        anvil_config.port = 8545;
        self.start_anvil(anvil_config).await?;

        // Wait for Anvil to be ready
        sleep(Duration::from_secs(3)).await;

        // Start Alto
        let mut alto_config = config.clone();
        alto_config.port = 4337;
        alto_config.rpc_url = "http://localhost:8545".to_string();
        self.alto_bundler.start(&alto_config, &self.state_file).await?;

        info!("✅ All bundler services started successfully with Alto!");
        info!("Anvil RPC: http://localhost:8545");
        info!("Alto Bundler RPC: http://localhost:4337");

        Ok(())
    }

    pub async fn stop(&mut self, name: &str) -> Result<()> {
        info!("Stopping {}...", name);

        if name == "rundler" {
            // Stop Docker containers
            self.rundler_bundler.stop_containers().await?;
        }

        // Try to kill by process name first
        let result = Command::new("pkill").arg("-f").arg(name).output();

        if let Ok(output) = result {
            if output.status.success() {
                info!("✅ {} stopped via pkill", name);
            } else {
                // Try to kill by PID if we have it in state
                if let Some(pid) = self.get_pid_from_state(name)? {
                    let _ = Command::new("kill").arg(pid.to_string()).output();
                    info!("Attempted to stop {} via PID {}", name, pid);
                }
            }
        }

        // Remove from state file
        self.remove_from_state(name)?;
        Ok(())
    }

    pub async fn stop_all(&mut self) -> Result<()> {
        info!("Stopping all bundler services...");

        // Stop managed processes
        let process_names: Vec<String> = self.processes.keys().cloned().collect();
        for name in process_names {
            self.stop(&name).await?;
        }

        // Kill any remaining processes
        let _ = Command::new("pkill").arg("-f").arg("anvil").output();
        let _ = Command::new("pkill").arg("-f").arg("rundler").output();

        // Stop Rundler containers
        let _ = self.rundler_bundler.stop_containers().await;

        // Clear state file
        if self.state_file.exists() {
            fs::remove_file(&self.state_file)?;
        }

        info!("✅ All bundler services stopped");
        Ok(())
    }

    pub async fn status(&self) -> Result<()> {
        info!("Checking bundler service status...");

        // Check Anvil
        let anvil_running = self
            .check_service("http://localhost:8545", "eth_blockNumber")
            .await;
        if anvil_running {
            info!("✅ Anvil is running on port 8545");
        } else {
            info!("❌ Anvil is not running");
        }

        // Check Rundler
        let rundler_running = self
            .check_service("http://localhost:4337", "eth_chainId")
            .await;
        if rundler_running {
            info!("✅ Rundler bundler is running on port 4337");
        } else {
            info!("❌ Rundler bundler is not running");
        }

        // Show state file info if it exists
        if self.state_file.exists() {
            if let Ok(contents) = fs::read_to_string(&self.state_file) {
                if let Ok(processes) = serde_json::from_str::<Vec<BundlerProcess>>(&contents) {
                    info!("📋 Managed processes:");
                    for process in processes {
                        let uptime = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                            - process.started_at;
                        info!(
                            "  - {} (PID: {}, Port: {}, Uptime: {}s)",
                            process.name, process.pid, process.port, uptime
                        );
                    }
                }
            }
        }

        Ok(())
    }

    async fn is_running(&self, name: &str) -> Result<bool> {
        match name {
            "anvil" => Ok(self
                .check_service("http://localhost:8545", "eth_blockNumber")
                .await),
            "rundler" => Ok(self
                .check_service("http://localhost:4337", "eth_chainId")
                .await),
            _ => Ok(false),
        }
    }

    async fn check_service(&self, url: &str, method: &str) -> bool {
        let payload = format!(
            r#"{{"jsonrpc":"2.0","method":"{}","params":[],"id":1}}"#,
            method
        );

        let result = Command::new("curl")
            .arg("-s")
            .arg("--connect-timeout")
            .arg("2")
            .arg(url)
            .arg("-X")
            .arg("POST")
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg(&payload)
            .output();

        match result {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    pub fn save_state(&self, process: &BundlerProcess) -> Result<()> {
        let mut processes = if self.state_file.exists() {
            let contents = fs::read_to_string(&self.state_file)?;
            serde_json::from_str::<Vec<BundlerProcess>>(&contents).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Remove existing process with same name
        processes.retain(|p| p.name != process.name);

        // Add new process
        processes.push(process.clone());

        let contents = serde_json::to_string_pretty(&processes)?;
        fs::write(&self.state_file, contents)?;

        Ok(())
    }

    fn get_pid_from_state(&self, name: &str) -> Result<Option<u32>> {
        if !self.state_file.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&self.state_file)?;
        let processes = serde_json::from_str::<Vec<BundlerProcess>>(&contents).unwrap_or_default();

        Ok(processes.iter().find(|p| p.name == name).map(|p| p.pid))
    }

    fn remove_from_state(&self, name: &str) -> Result<()> {
        if !self.state_file.exists() {
            return Ok(());
        }

        let contents = fs::read_to_string(&self.state_file)?;
        let mut processes =
            serde_json::from_str::<Vec<BundlerProcess>>(&contents).unwrap_or_default();

        processes.retain(|p| p.name != name);

        let contents = serde_json::to_string_pretty(&processes)?;
        fs::write(&self.state_file, contents)?;

        Ok(())
    }
}
