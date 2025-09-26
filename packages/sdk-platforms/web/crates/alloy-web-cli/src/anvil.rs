use eyre::{Result, eyre};
use std::process::Command;
use tokio::time::Duration;
use tracing::{info, warn};

pub struct AnvilManager;

impl AnvilManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self, fork_url: Option<String>) -> Result<()> {
        info!("Starting Anvil...");

        // Kill any existing Anvil process
        Command::new("pkill").args(["-f", "anvil"]).output().ok();

        // Wait for process to die
        tokio::time::sleep(Duration::from_millis(500)).await;

        let mut args = vec![
            "--port",
            "8545",
            "--accounts",
            "10",
            "--balance",
            "10000",
            "--mnemonic",
            "test test test test test test test test test test test junk",
            "--block-time",
            "1",
        ];

        let fork_string;
        if let Some(url) = fork_url {
            fork_string = url;
            args.push("--fork-url");
            args.push(&fork_string);
        }

        let child = Command::new("anvil")
            .args(&args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| eyre!("Failed to start Anvil: {}", e))?;

        // Wait for Anvil to be ready
        for i in 0..30 {
            tokio::time::sleep(Duration::from_millis(1000)).await;

            let output = Command::new("curl")
                .args([
                    "-s",
                    "http://localhost:8545",
                    "-X",
                    "POST",
                    "-H",
                    "Content-Type: application/json",
                    "-d",
                    r#"{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}"#,
                ])
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let response = String::from_utf8_lossy(&output.stdout);
                    if response.contains("result") {
                        info!("✅ Anvil started successfully on port 8545");

                        // Detach the process so it continues running
                        let _ = child.id();
                        std::mem::forget(child);

                        return Ok(());
                    }
                }
            }

            if i == 29 {
                return Err(eyre!("Anvil failed to start after 30 seconds"));
            }
        }

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Anvil...");

        let output = Command::new("pkill")
            .args(["-f", "anvil"])
            .output()
            .map_err(|e| eyre!("Failed to kill Anvil: {}", e))?;

        if output.status.success() {
            info!("✅ Anvil stop signal sent");
        } else {
            warn!("⚠️ Anvil was not running or could not be stopped");
        }

        // Wait for the process to actually exit
        let max_wait_secs = 10;
        let poll_interval_ms = 500;
        let start = std::time::Instant::now();
        let max_duration = Duration::from_secs(max_wait_secs);

        info!("⏳ Waiting for Anvil process to exit...");

        while start.elapsed() < max_duration {
            if !self.is_running().await {
                let elapsed = start.elapsed().as_secs_f64();
                info!("✅ Anvil process exited after {:.1} seconds", elapsed);
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(poll_interval_ms)).await;
        }

        // If we get here, the process didn't exit within the timeout
        warn!(
            "⚠️ Anvil process did not exit within {} seconds",
            max_wait_secs
        );
        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        let output = Command::new("pgrep").args(["-f", "anvil"]).output();

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    pub async fn status(&self) -> Result<()> {
        let is_running = self.is_running().await;

        if is_running {
            info!("✅ Anvil: Running on port 8545");
        } else {
            info!("❌ Anvil: Not running");
        }

        Ok(())
    }
}
