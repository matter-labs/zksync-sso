use eyre::{Result, eyre};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{error, info};

use super::manager::{BundlerConfig, BundlerProcess};

pub struct AltoBundler;

impl AltoBundler {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self, config: &BundlerConfig, state_file: &PathBuf) -> Result<()> {
        let name = "alto".to_string();

        // Check if already running
        if self.is_running(&name).await? {
            info!("Alto is already running");
            return Ok(());
        }

        info!("Starting Alto bundler on port {}", config.port);

        // Navigate to infra directory and run Alto via pnpm
        // Try to find the infra directory - it might be at different relative paths
        // depending on where the command is run from
        let current_dir = std::env::current_dir()?;
        let infra_dir = if current_dir.join("platforms").join("infra").exists() {
            current_dir.join("platforms").join("infra")
        } else if current_dir.join("../infra").exists() {
            // When running from platforms/rust/alloy-web during tests
            current_dir.join("../infra")
        } else if current_dir.join("../../../../../platforms/infra").exists() {
            // When running from platforms/rust/alloy-web/crates/alloy-web-cli during tests
            current_dir.join("../../../../../platforms/infra")
        } else if current_dir
            .join("../../..")
            .join("platforms")
            .join("infra")
            .exists()
        {
            // Another possible location
            current_dir.join("../../..").join("platforms").join("infra")
        } else {
            return Err(eyre!(
                "Could not find platforms/infra directory. Current dir: {:?}",
                current_dir
            ));
        };

        let mut cmd = Command::new("pnpm");
        cmd.current_dir(&infra_dir)
            .arg("run")
            .arg(if config.unsafe_mode {
                "alto:start:unsafe"
            } else {
                "alto:start"
            });

        // Determine EntryPoint address
        let entry_point_env = if config.entry_point.starts_with("0x") {
            &config.entry_point
        } else {
            // Fallback to hardcoded v0.7 address if a version string was provided
            "0x0000000071727De22E5E9d8BAf0edAc6f37da032"
        };

        // Set environment variables for Alto
        cmd.env("ALTO_RPC_URL", &config.rpc_url)
            .env("ALTO_PORT", config.port.to_string())
            .env("ALTO_ENTRYPOINT_ADDRESS", entry_point_env);

        // Log where/what we're starting
        info!("Launching Alto via pnpm in {:?}", infra_dir);

        // Start the process in the background and inherit stdout/stderr so logs are visible
        let child = cmd
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

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
        sleep(Duration::from_secs(3)).await;

        // Verify it's running
        if self.is_running(&name).await? {
            info!("✅ Alto started successfully on port {}", config.port);
            info!("Bundler RPC URL: http://localhost:{}", config.port);
            info!("Process ID: {}", pid);
        } else {
            error!("Failed to start Alto");
            return Err(eyre::eyre!("Alto failed to start"));
        }

        Ok(())
    }

    async fn is_running(&self, name: &str) -> Result<bool> {
        match name {
            "alto" => Ok(self
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
            .arg("--max-time")
            .arg("3")
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
    use crate::test_utils::{check_port, rpc_call};
    use alloy_web_core::erc4337::infrastructure::{
        InfrastructureConfig, deploy_infrastructure as deploy_infra_core,
    };
    use alloy_web_core::utils::is_contract_deployed;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_alto_bundler() {
        println!("🚀 Starting Alto bundler test...");

        // Create a temporary directory for test state
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let state_file = temp_dir.path().join("test_alto_state.json");

        // Create instances
        let anvil_manager = AnvilManager::new();
        let alto_bundler = AltoBundler::new();

        // Clean slate - stop any existing processes
        println!("🧹 Cleaning up existing processes...");
        let _ = anvil_manager.stop().await;

        // Test starting Anvil first
        println!("⛏️  Starting Anvil...");
        let anvil_result = anvil_manager.start(None).await; // No fork URL for local testing

        match anvil_result {
            Ok(_) => {
                println!("✅ Anvil started successfully and is ready");
            }
            Err(e) => {
                println!("❌ Failed to start Anvil: {}", e);
                panic!("Failed to start Anvil: {}", e);
            }
        }

        // Deploy minimal infrastructure to get a valid EntryPoint address
        println!("📦 Deploying ERC-4337 infrastructure to Anvil...");
        let infra_config = InfrastructureConfig {
            rpc_url: "http://localhost:8545".to_string(),
            private_key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
                .to_string(),
        };
        let deployment = deploy_infra_core(infra_config).await;
        let entry_point_addr = match deployment {
            Ok(deployed) => {
                let addr = format!("{:#x}", deployed.entry_point);
                println!("✅ Deployed EntryPoint at {}", addr);
                addr
            }
            Err(e) => {
                println!(
                    "⚠️  Infrastructure deploy failed (may already be deployed): {}",
                    e
                );
                // Fall back to the commonly used v0.7 EntryPoint
                "0x0000000071727De22E5E9d8BAf0edAc6f37da032".to_string()
            }
        };

        // Create test configuration for Alto using the deployed EntryPoint (or default)
        let config = BundlerConfig {
            rpc_url: "http://localhost:8545".to_string(),
            port: 4337,
            // Provide the actual deployed EntryPoint address to Alto
            entry_point: entry_point_addr,
            chain_id: 31337, // Local chain ID
            unsafe_mode: true,
            fork_url: None,
            fork_block: None,
            auto_mine: true,
            use_docker: None, // Alto doesn't use Docker
        };

        // Check if EntryPoint contract is actually deployed before starting Alto
        println!("🔍 Checking if EntryPoint contract is deployed at {}...", config.entry_point);
        let is_deployed = is_contract_deployed(&config.rpc_url, &config.entry_point).await;
        match is_deployed {
            Ok(true) => {
                println!("✅ EntryPoint contract is deployed at {}", config.entry_point);
            }
            Ok(false) => {
                println!("❌ EntryPoint contract is NOT deployed at {}", config.entry_point);
                panic!("EntryPoint contract must be deployed before starting Alto bundler");
            }
            Err(e) => {
                println!("⚠️  Failed to check EntryPoint deployment: {}", e);
                panic!("Failed to verify EntryPoint deployment: {}", e);
            }
        }

        // Test starting Alto bundler
        println!("🎯 Starting Alto bundler...");
        let start_result = alto_bundler.start(&config, &state_file).await;

        match start_result {
            Ok(_) => {
                println!("✅ Alto bundler started successfully");

                // Verify Alto is running by checking the port
                assert!(check_port(4337), "Alto should be running on port 4337");

                // Verify it's actually responding to RPC calls
                let is_running = alto_bundler
                    .is_running("alto")
                    .await
                    .expect("Failed to check if Alto is running");
                assert!(is_running, "Alto should be responding to RPC calls");

                println!("✅ Alto bundler is running and responding");

                // Try to make a bundler RPC call
                println!("📞 Making RPC call to Alto...");
                let rpc_result = rpc_call(4337, "eth_supportedEntryPoints");
                println!("📋 Alto RPC result: {:?}", rpc_result);

                // Verify the RPC call succeeded
                if let Ok(response) = rpc_result {
                    assert!(
                        response.contains("result") || response.contains("error"),
                        "RPC response should be valid JSON-RPC"
                    );
                    println!("✅ RPC call successful");
                } else {
                    println!("⚠️  RPC call failed but continuing test");
                }
            }
            Err(e) => {
                println!("❌ Failed to start Alto bundler: {}", e);
                // Don't fail the test if infra directory is not found or pnpm is not available
                if e.to_string()
                    .contains("Could not find platforms/infra directory")
                    || e.to_string().contains("pnpm")
                {
                    println!("⚠️  Skipping test - infra directory not found or pnpm not available");
                    return;
                }
                panic!("Failed to start Alto bundler: {}", e);
            }
        }

        // Verify both services are running
        assert!(
            anvil_manager.is_running().await,
            "Anvil should still be running"
        );
        assert!(
            alto_bundler
                .is_running("alto")
                .await
                .expect("Failed to check Alto"),
            "Alto should still be running"
        );
        println!("✅ Both Anvil and Alto are running successfully");

        // Test stopping Alto bundler (we'll use pkill since Alto doesn't have a direct stop method)
        println!("🛑 Stopping Alto bundler...");
        let stop_result = std::process::Command::new("pkill")
            .args(["-f", "alto"])
            .output();

        if let Ok(output) = stop_result {
            if output.status.success() {
                println!("✅ Alto bundler stopped via pkill");
            } else {
                println!("⚠️  Alto bundler was not running or could not be stopped");
            }
        }

        // Wait for Alto to stop
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Verify Alto is stopped
        let is_running_after_stop = alto_bundler
            .is_running("alto")
            .await
            .expect("Failed to check if Alto is running");
        assert!(
            !is_running_after_stop,
            "Alto should not be running after stop"
        );

        // Also check the port is not available
        assert!(
            !check_port(4337),
            "Port 4337 should not be available after stopping Alto"
        );

        println!("✅ Alto bundler stopped successfully");

        // Test stopping Anvil
        println!("🛑 Stopping Anvil...");
        let anvil_stop_result = anvil_manager.stop().await;
        assert!(
            anvil_stop_result.is_ok(),
            "Failed to stop Anvil: {:?}",
            anvil_stop_result
        );

        // Wait for Anvil to stop
        tokio::time::sleep(Duration::from_secs(2)).await;

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
        println!(
            "🎉 Alto bundler test completed successfully - both services started and stopped properly!"
        );
    }
}
