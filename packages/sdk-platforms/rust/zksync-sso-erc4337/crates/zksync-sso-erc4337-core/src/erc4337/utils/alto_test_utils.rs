use crate::erc4337::entry_point::config::EntryPointConfig;
use libc::{SIGINT, SIGTERM, kill as libc_kill, pid_t};
use std::{
    net::TcpListener,
    path::{Path, PathBuf},
    process::Stdio,
};
use strip_ansi_escapes::strip as strip_ansi_bytes;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::oneshot,
    time::{Duration, timeout},
};
use url::Url;

#[derive(Debug, Clone)]
pub enum BundlerStatus {
    Running,
    Stopped,
}

#[derive(Debug, Clone)]
pub struct AltoTestHelperConfig {
    pub entrypoint: EntryPointConfig,
    pub port: Option<u16>,
    pub node_url: Url,
    pub safe_mode: bool,
    pub executor_private_keys: Vec<String>,
    pub utility_private_key: String,
}

// impl Default for AltoTestHelperConfig {
//     fn default() -> Self {
//         Self {
//             entrypoint: EntryPointConfig::default(),
//             port: None,
//             node_url: Url::parse("http://127.0.0.1:8545").unwrap(),
//             safe_mode: false,
//             executor_private_keys: vec!["0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()],
//             utility_private_key: "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d".to_string(),
//         }
//     }
// }

impl AltoTestHelperConfig {
    pub fn default_ethereum() -> Self {
        Self {
          entrypoint: EntryPointConfig::default_ethereum(),
          port: None,
          node_url: Url::parse("http://127.0.0.1:8545").unwrap(),
          safe_mode: false,
          executor_private_keys: vec!["0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()],
          utility_private_key: "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d".to_string(),
      }
    }

    pub fn default_zksyncos() -> Self {
        Self {
            entrypoint: EntryPointConfig::default_zksyncos(),
            port: None,
            node_url: Url::parse("http://127.0.0.1:3050").unwrap(),
            safe_mode: false,
            executor_private_keys: vec!["0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110".to_string()],
            utility_private_key: "0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110".to_string(),
        }
    }
}

pub struct AltoTestHelper {
    config: AltoTestHelperConfig,
    contracts_dir: PathBuf,
    port: u16,
    child: Option<Child>,
}

impl AltoTestHelper {
    pub fn new(mut config: AltoTestHelperConfig) -> Self {
        let contracts_dir = resolve_contracts_dir();
        let port = config
            .port
            .unwrap_or_else(|| random_available_port().unwrap_or_else(|err| {
                eprintln!(
                    "[alto helper] failed to allocate random port: {err}; falling back to 4337"
                );
                4337
            }));
        config.port = Some(port);
        Self { config, contracts_dir, port, child: None }
    }

    pub fn contracts_dir(&self) -> &Path {
        &self.contracts_dir
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn url(&self) -> Url {
        Url::parse(&format!("http://127.0.0.1:{}", self.port))
            .expect("failed to parse bundler URL")
    }

    pub fn bundler_client(
        &self,
    ) -> crate::erc4337::bundler::pimlico::client::BundlerClient {
        use crate::erc4337::bundler::{
            config::BundlerConfig, pimlico::client::BundlerClient,
        };
        let bundler_url = format!("http://127.0.0.1:{}", self.port);
        let config = BundlerConfig::new(bundler_url);
        BundlerClient::new(config)
    }

    pub async fn start(&mut self) -> eyre::Result<()> {
        if self.child.is_some() {
            return Ok(());
        }

        let mut cmd = Command::new("pnpm");
        let entrypoints = self.config.entrypoint.address.to_string();
        let port = self.port.to_string();
        let rpc_url = self.config.node_url.to_string();
        let executor_private_keys =
            self.config
                .executor_private_keys
                .first()
                .ok_or_else(|| eyre::eyre!("missing executor private key"))?;
        let utility_private_key = self.config.utility_private_key.clone();

        cmd.current_dir(&self.contracts_dir)
            .arg("exec")
            .arg("alto")
            .arg("--entrypoints")
            .arg(&entrypoints)
            .arg("--port")
            .arg(&port)
            .arg("--rpc-url")
            .arg(&rpc_url)
            .arg("--executor-private-keys")
            .arg(executor_private_keys)
            .arg("--utility-private-key")
            .arg(&utility_private_key);

        if self.config.safe_mode {
            cmd.arg("--safe-mode");
        } else {
            cmd.arg("--no-safe-mode");
        }

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        let (ready_tx, ready_rx) = oneshot::channel::<()>();
        let mut ready_tx_opt = Some(ready_tx);

        let bundler_port = self.port;
        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout).lines();
            let mut ready_tx_stdout = ready_tx_opt.take();
            let port = bundler_port;
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    let clean = strip_ansi(&line);
                    let ready = is_alto_ready_line(&line, port);
                    println!("[alto stdout] {}", line);
                    println!(
                        "[alto stdout dbg] ready={} clean=\"{}\"",
                        ready, clean
                    );
                    if ready && let Some(tx) = ready_tx_stdout.take() {
                        let _ = tx.send(());
                    }
                }
                if let Some(tx) = ready_tx_stdout.take() {
                    let _ = tx.send(());
                }
            });
        } else if let Some(tx) = ready_tx_opt.take() {
            let _ = tx.send(());
        }

        if let Some(stderr) = child.stderr.take() {
            let mut reader = BufReader::new(stderr).lines();
            let port = bundler_port;
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    let clean = strip_ansi(&line);
                    let ready = is_alto_ready_line(&line, port);
                    eprintln!("[alto stderr] {}", line);
                    eprintln!(
                        "[alto stderr dbg] ready={} clean=\"{}\"",
                        ready, clean
                    );
                }
            });
        }

        println!("[alto helper] waiting for readiness signal...");
        match timeout(Duration::from_secs(60), ready_rx).await {
            Ok(Ok(())) => {
                println!("[alto helper] readiness signal received.");
            }
            _ => {
                _ = child.kill();
                let _ = child.wait().await;
                return Err(eyre::eyre!(format!(
                    "Timed out waiting for Alto readiness (expected 'Server listening at http://0.0.0.0:{}')",
                    bundler_port
                )));
            }
        }

        println!(
            "[alto helper] storing bundler child and returning from start()."
        );
        self.child = Some(child);
        Ok(())
    }

    pub async fn stop(&mut self) -> eyre::Result<()> {
        if let Some(mut child) = self.child.take() {
            println!("[alto helper] stopping bundler...");
            terminate_child_gracefully(&mut child, "bundler").await;
        }
        Ok(())
    }

    pub async fn status(&mut self) -> eyre::Result<BundlerStatus> {
        if let Some(child) = self.child.as_mut() {
            match child.try_wait()? {
                None => Ok(BundlerStatus::Running),
                Some(_exit) => Ok(BundlerStatus::Stopped),
            }
        } else {
            Ok(BundlerStatus::Stopped)
        }
    }
}

impl Drop for AltoTestHelper {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            println!(
                "[alto helper] dropping AltoTestHelper, stopping bundler..."
            );

            if let Some(id) = child.id() {
                unsafe {
                    use libc::{SIGINT, kill as libc_kill, pid_t};
                    let _ = libc_kill(id as pid_t, SIGINT);
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = child.start_kill();
        }
    }
}

fn is_alto_ready_line(line: &str, port: u16) -> bool {
    let clean = strip_ansi(line);
    let scheme = "http";
    let host = "0.0.0.0";
    let scheme_host_port = format!("{}://{}:{}", scheme, host, port);
    clean.contains(&format!("Server listening at {}", scheme_host_port))
}

fn strip_ansi(input: &str) -> String {
    let bytes = strip_ansi_bytes(input.as_bytes());
    String::from_utf8_lossy(&bytes).into_owned()
}

fn random_available_port() -> std::io::Result<u16> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let port = listener.local_addr()?.port();
    Ok(port)
}

pub(super) fn resolve_contracts_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("ZKSSO_ERC4337_CONTRACTS_DIR") {
        let p = PathBuf::from(dir);
        if p.exists() {
            return p;
        }
    }
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let candidate =
        Path::new(&manifest_dir).join("../../../../../erc4337-contracts");
    candidate.canonicalize().unwrap_or_else(|_| candidate.to_path_buf())
}

pub(super) async fn terminate_child_gracefully(child: &mut Child, label: &str) {
    if let Ok(Some(status)) = child.try_wait() {
        println!("[alto helper] {label} already exited: {status}");
        return;
    }

    if let Some(id) = child.id() {
        unsafe {
            let _ = libc_kill(id as pid_t, SIGINT);
        }
        if let Ok(Ok(status)) =
            timeout(Duration::from_secs(2), child.wait()).await
        {
            println!("[alto helper] {label} exited after SIGINT: {status}");
            return;
        }

        unsafe {
            let _ = libc_kill(id as pid_t, SIGTERM);
        }
        if let Ok(Ok(status)) =
            timeout(Duration::from_secs(2), child.wait()).await
        {
            println!("[alto helper] {label} exited after SIGTERM: {status}");
            return;
        }
    }

    _ = child.kill();
    match timeout(Duration::from_secs(10), child.wait()).await {
        Ok(Ok(status)) => {
            println!("[alto helper] {label} exited after kill(): {status}")
        }
        Ok(Err(e)) => eprintln!("[alto helper] error waiting for {label}: {e}"),
        Err(_) => {
            eprintln!("[alto helper] timeout waiting for {label} to exit")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::alloy_utilities::test_utilities::start_node_and_deploy_contracts;
    use eyre::Result;

    #[tokio::test]
    #[ignore = "manual test"]
    async fn test_alto_start_stop_without_anviltesthelper() -> Result<()> {
        let (node_url, anvil_instance, _, _, _) =
            start_node_and_deploy_contracts().await?;

        let alto_cfg = AltoTestHelperConfig {
            node_url,
            ..AltoTestHelperConfig::default_ethereum()
        };

        let mut alto = AltoTestHelper::new(alto_cfg);
        println!("[test] calling alto.start()...");
        alto.start().await?;
        println!("[test] alto.start() returned");
        println!("[test] checking status...");
        assert!(matches!(alto.status().await?, BundlerStatus::Running));
        println!("[test] status is Running");
        println!("[test] calling alto.stop()...");
        alto.stop().await?;
        println!("[test] alto.stop() returned");
        println!("[test] stopping anvil...");
        drop(anvil_instance);
        println!("[test] checking status stopped...");
        assert!(matches!(alto.status().await?, BundlerStatus::Stopped));
        println!("[test] status is Stopped");

        Ok(())
    }
}
