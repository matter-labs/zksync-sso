use eyre::Result;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{error, info};

use super::manager::{BundlerConfig, BundlerProcess};

pub struct RundlerBundler;

impl RundlerBundler {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self, config: &BundlerConfig, state_file: &PathBuf) -> Result<()> {
        let name = "rundler".to_string();

        // Check if already running
        if self.is_running(&name).await? {
            info!("Rundler bundler is already running");
            return Ok(());
        }

        info!("Starting Rundler bundler on port {}", config.port);

        // Set EntryPoint address based on version
        let entry_point_addr = match config.entry_point.as_str() {
            "v0.6" | "0.6" => "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
            "v0.7" | "0.7" => "0x0000000071727De22E5E9d8BAf0edAc6f37da032",
            _ => {
                error!("Unknown EntryPoint version: {}", config.entry_point);
                return Err(eyre::eyre!(
                    "Unknown EntryPoint version: {}",
                    config.entry_point
                ));
            }
        };

        let child = if config.use_docker.unwrap_or(false) {
            // Use Docker/OrbStack
            self.start_rundler_docker(config.clone(), entry_point_addr)
                .await?
        } else {
            // Use local binary
            self.start_rundler_binary(config.clone(), entry_point_addr)
                .await?
        };

        let pid = child.id();

        // Save state
        let process_info = BundlerProcess {
            pid,
            name: name.clone(),
            port: config.port,
            rpc_url: format!("http://localhost:{}", config.port),
            started_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            config: config.clone(),
        };

        self.save_state(&process_info, state_file)?;

        // Wait a moment for the process to start
        sleep(Duration::from_secs(5)).await;

        // Verify it's running
        if self.is_running(&name).await? {
            info!(
                "✅ Rundler bundler started successfully on port {}",
                config.port
            );
            info!("Bundler RPC URL: http://localhost:{}", config.port);
            info!("EntryPoint: {} ({})", config.entry_point, entry_point_addr);
        } else {
            error!("Failed to start Rundler bundler");
            return Err(eyre::eyre!("Rundler bundler failed to start"));
        }

        Ok(())
    }

    async fn start_rundler_binary(
        &self,
        config: BundlerConfig,
        entry_point_addr: &str,
    ) -> Result<Child> {
        // First check if rundler is installed
        let check = Command::new("which").arg("rundler").output();

        if check.is_err() || !check.unwrap().status.success() {
            error!("Rundler is not installed. Please install it first:");
            error!("cargo install --git https://github.com/alchemyplatform/rundler");
            return Err(eyre::eyre!("Rundler is not installed"));
        }

        let mut cmd = Command::new("rundler");

        // Basic configuration
        cmd.arg("--rpc-url").arg(&config.rpc_url);
        cmd.arg("--port").arg(config.port.to_string());
        cmd.arg("--chain-id").arg(config.chain_id.to_string());
        cmd.arg("--entry-points").arg(entry_point_addr);

        if config.unsafe_mode {
            cmd.arg("--unsafe");
        }

        // Start the process in the background
        let child = cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;

        Ok(child)
    }

    async fn start_rundler_docker(
        &self,
        config: BundlerConfig,
        _entry_point_addr: &str,
    ) -> Result<Child> {
        // Check if Docker is available
        let docker_check = Command::new("docker").arg("--version").output();
        let orb_check = Command::new("orb").arg("status").output();

        let use_orb = docker_check.is_err() && orb_check.is_ok();
        let docker_cmd = if use_orb { "orb" } else { "docker" };

        info!("Using {} to run Rundler", docker_cmd);

        // First, try to pull the official Rundler image
        self.ensure_rundler_image(docker_cmd).await?;

        // Build the Docker command based on the documentation
        let mut cmd = Command::new(docker_cmd);

        if use_orb {
            cmd.arg("docker");
        }

        cmd.arg("run")
            .arg("-d")
            .arg("--name")
            .arg("rundler-container")
            .arg("--network")
            .arg("host") // Use host network so container can access localhost:8545
            .arg("-e")
            .arg("RUST_LOG=INFO")
            .arg("-e")
            .arg("NETWORK=ethereum_sepolia")
            .arg("-e")
            .arg(format!("NODE_HTTP={}", config.rpc_url))
            .arg("-e")
            .arg("DISABLE_ENTRY_POINT_V0_6=true") // Only use v0.7
            .arg("-e")
            .arg("BUILDER_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80") // Test private key
            .arg("-e")
            .arg(format!("RPC_PORT={}", config.port)) // Use our desired port
            .arg("-e")
            .arg("RPC_HOST=0.0.0.0"); // Allow external connections

        if config.unsafe_mode {
            cmd.arg("-e").arg("UNSAFE=true");
        }

        // Use the official Rundler image with the node command
        cmd.arg("alchemyplatform/rundler:latest").arg("node");

        info!("Executing Docker command: {:?}", cmd);

        // Start the container
        let child = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                error!("Failed to start Rundler container: {}", e);
                eyre::eyre!("Failed to start Rundler container: {}", e)
            })?;

        Ok(child)
    }

    async fn ensure_rundler_image(&self, docker_cmd: &str) -> Result<()> {
        info!("Ensuring Rundler Docker image is available...");

        // Check if the official image already exists
        let mut check_cmd = Command::new(docker_cmd);
        if docker_cmd == "orb" {
            check_cmd.arg("docker");
        }
        check_cmd
            .arg("images")
            .arg("alchemyplatform/rundler:latest");

        let check_output = check_cmd.output()?;
        if check_output.status.success() {
            let output_str = String::from_utf8_lossy(&check_output.stdout);
            if output_str.contains("alchemyplatform/rundler") && output_str.lines().count() > 1 {
                info!("Rundler image already exists");
                return Ok(());
            }
        }

        // Pull the official image from Docker Hub
        info!("Pulling official Rundler image from Docker Hub...");
        let mut pull_cmd = Command::new(docker_cmd);
        if docker_cmd == "orb" {
            pull_cmd.arg("docker");
        }

        pull_cmd.arg("pull").arg("alchemyplatform/rundler:latest");

        let pull_output = pull_cmd.output()?;
        if !pull_output.status.success() {
            error!("Failed to pull Rundler image");
            error!(
                "Pull output: {}",
                String::from_utf8_lossy(&pull_output.stderr)
            );
            return Err(eyre::eyre!(
                "Failed to pull Rundler Docker image from Docker Hub"
            ));
        }

        info!("✅ Rundler Docker image pulled successfully");
        Ok(())
    }

    pub async fn stop_containers(&self) -> Result<()> {
        // Try Docker first
        let docker_stop = Command::new("docker")
            .args(["stop", "rundler-container"])
            .output();

        let docker_rm = Command::new("docker")
            .args(["rm", "rundler-container"])
            .output();

        if docker_stop.is_ok() && docker_rm.is_ok() {
            info!("✅ Rundler Docker container stopped and removed");
            return Ok(());
        }

        // Try OrbStack
        let orb_stop = Command::new("orb")
            .args(["docker", "stop", "rundler-container"])
            .output();

        let orb_rm = Command::new("orb")
            .args(["docker", "rm", "rundler-container"])
            .output();

        if orb_stop.is_ok() && orb_rm.is_ok() {
            info!("✅ Rundler OrbStack container stopped and removed");
            return Ok(());
        }

        info!("No Rundler containers found to stop");
        Ok(())
    }

    async fn is_running(&self, name: &str) -> Result<bool> {
        match name {
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

    fn save_state(&self, process: &BundlerProcess, state_file: &PathBuf) -> Result<()> {
        use serde_json;
        use std::fs;

        let mut processes = if state_file.exists() {
            let contents = fs::read_to_string(state_file)?;
            serde_json::from_str::<Vec<BundlerProcess>>(&contents).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Remove existing process with same name
        processes.retain(|p| p.name != process.name);

        // Add new process
        processes.push(process.clone());

        let contents = serde_json::to_string_pretty(&processes)?;
        fs::write(state_file, contents)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anvil::AnvilManager;
    use crate::test_utils::check_port;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_rundler_bundler() {
        // Create a temporary directory for test state
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let state_file = temp_dir.path().join("test_bundler_state.json");

        // Create instances
        let anvil_manager = AnvilManager::new();
        let bundler = RundlerBundler::new();

        // Create test configuration
        let config = BundlerConfig {
            rpc_url: "http://localhost:8545".to_string(),
            port: 4337,
            entry_point: "v0.7".to_string(),
            chain_id: 31337, // Local chain ID
            unsafe_mode: true,
            fork_url: None,
            fork_block: None,
            auto_mine: true,
            use_docker: Some(false), // Use binary for testing
        };

        // Clean slate - stop any existing processes
        println!("Cleaning up any existing processes...");
        let _ = anvil_manager.stop().await;
        let _ = bundler.stop_containers().await;

        // Test starting Anvil first
        println!("Starting Anvil...");
        let anvil_result = anvil_manager.start(None).await; // No fork URL for local testing

        match anvil_result {
            Ok(_) => {
                println!("✅ Anvil started successfully");

                // Verify Anvil is running
                assert!(check_port(8545), "Anvil should be running on port 8545");
                assert!(
                    anvil_manager.is_running().await,
                    "Anvil should be responding"
                );

                println!("✅ Anvil is running and responding");
            }
            Err(e) => {
                println!("❌ Failed to start Anvil: {}", e);
                panic!("Failed to start Anvil: {}", e);
            }
        }

        // Test starting Rundler bundler
        println!("Starting Rundler bundler...");
        let start_result = bundler.start(&config, &state_file).await;

        match start_result {
            Ok(_) => {
                println!("✅ Rundler bundler started successfully");

                // Verify Rundler is running by checking the port
                assert!(check_port(4337), "Rundler should be running on port 4337");

                // Verify it's actually responding to RPC calls
                let is_running = bundler
                    .is_running("rundler")
                    .await
                    .expect("Failed to check if Rundler is running");
                assert!(is_running, "Rundler should be responding to RPC calls");

                println!("✅ Rundler bundler is running and responding");
            }
            Err(e) => {
                println!("❌ Failed to start Rundler bundler: {}", e);
                // Don't fail the test if Rundler binary is not installed
                if e.to_string().contains("not installed") {
                    println!("⚠️  Skipping test - Rundler binary not installed");
                    return;
                }
                panic!("Failed to start Rundler bundler: {}", e);
            }
        }

        // Verify both services are running
        assert!(
            anvil_manager.is_running().await,
            "Anvil should still be running"
        );
        assert!(
            bundler
                .is_running("rundler")
                .await
                .expect("Failed to check Rundler"),
            "Rundler should still be running"
        );
        println!("✅ Both Anvil and Rundler are running successfully");

        // Test stopping Rundler bundler
        println!("Stopping Rundler bundler...");
        let stop_result = bundler.stop_containers().await;
        assert!(
            stop_result.is_ok(),
            "Failed to stop Rundler containers: {:?}",
            stop_result
        );

        // Wait a moment for the process to actually stop
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Verify Rundler is stopped
        let is_running_after_stop = bundler
            .is_running("rundler")
            .await
            .expect("Failed to check if Rundler is running");
        assert!(
            !is_running_after_stop,
            "Rundler should not be running after stop"
        );

        // Also check the port is not available
        assert!(
            !check_port(4337),
            "Port 4337 should not be available after stopping Rundler"
        );

        println!("✅ Rundler bundler stopped successfully");

        // Test stopping Anvil
        println!("Stopping Anvil...");
        let anvil_stop_result = anvil_manager.stop().await;
        assert!(
            anvil_stop_result.is_ok(),
            "Failed to stop Anvil: {:?}",
            anvil_stop_result
        );

        // Verify Anvil is stopped
        assert!(
            !anvil_manager.is_running().await,
            "Anvil should not be running after stop"
        );
        assert!(
            !check_port(8545),
            "Port 8545 should not be available after stopping Anvil"
        );

        println!("✅ Anvil stopped successfully");
        println!("✅ Test completed successfully - both services started and stopped properly");
    }
}
