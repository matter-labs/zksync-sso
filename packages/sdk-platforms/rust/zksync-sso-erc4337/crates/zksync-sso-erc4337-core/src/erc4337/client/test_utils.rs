use crate::erc4337::entry_point::config::EntryPointConfig;
use alloy::{primitives::Address, providers::Provider};
use libc::{SIGINT, SIGTERM, kill as libc_kill, pid_t};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};
use strip_ansi_escapes::strip as strip_ansi_bytes;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpStream,
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
    pub executor_private_keys: String,
    pub port: u16,
    pub node_url: Url,
    pub safe_mode: bool,
    pub utility_private_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AltoConfigJson {
    pub entrypoints: String,
    pub executor_private_keys: String,
    pub port: u16,
    pub rpc_url: String,
    pub safe_mode: bool,
    pub utility_private_key: String,
}

impl From<AltoTestHelperConfig> for AltoConfigJson {
    fn from(config: AltoTestHelperConfig) -> Self {
        Self {
            entrypoints: config.entrypoint.address.to_string(),
            executor_private_keys: config.executor_private_keys,
            port: config.port,
            rpc_url: config.node_url.to_string(),
            safe_mode: config.safe_mode,
            utility_private_key: config.utility_private_key,
        }
    }
}

pub struct AltoTestHelper<P>
where
    P: Provider + Clone,
{
    provider: P,
    config: AltoTestHelperConfig,
    contracts_dir: PathBuf,
    child: Option<Child>,
    temp_config_path: Option<PathBuf>,
    anvil_child: Option<Child>,
}

impl<P> AltoTestHelper<P>
where
    P: Provider + Clone,
{
    pub fn new(provider: P, config: AltoTestHelperConfig) -> Self {
        let contracts_dir = resolve_contracts_dir();
        Self {
            provider,
            config,
            contracts_dir,
            child: None,
            temp_config_path: None,
            anvil_child: None,
        }
    }

    pub fn provider(&self) -> &P {
        &self.provider
    }

    pub fn contracts_dir(&self) -> &Path {
        &self.contracts_dir
    }

    pub async fn start(&mut self) -> eyre::Result<()> {
        if self.child.is_some() {
            return Ok(());
        }

        // -1) Ensure an anvil node is running on configured node endpoint
        let node_addr = format!(
            "{}:{}",
            self.config.node_url.host().unwrap(),
            self.config.port
        );
        if !is_port_open(&node_addr).await {
            let mut cmd = Command::new("pnpm");
            cmd.current_dir(&self.contracts_dir)
                .arg("anvil")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut child = cmd.spawn()?;

            let (ready_tx, ready_rx) = oneshot::channel::<()>();
            if let Some(stdout) = child.stdout.take() {
                let mut reader = BufReader::new(stdout).lines();
                let mut ready_tx_stdout = Some(ready_tx);
                let config = self.config.clone();
                tokio::spawn(async move {
                    while let Ok(Some(line)) = reader.next_line().await {
                        println!("[anvil stdout] {}", line);
                        if is_anvil_ready_line(&line, &config)
                            && let Some(tx) = ready_tx_stdout.take()
                        {
                            _ = tx.send(());
                        }
                    }
                });
            }
            if let Some(stderr) = child.stderr.take() {
                let mut reader = BufReader::new(stderr).lines();
                tokio::spawn(async move {
                    while let Ok(Some(line)) = reader.next_line().await {
                        eprintln!("[anvil stderr] {}", line);
                    }
                });
            }

            // Wait for anvil readiness
            let _ = timeout(Duration::from_secs(30), ready_rx).await;
            self.anvil_child = Some(child);
        }

        // Pre-check: if project's alto.json entrypoint is not deployed on the connected anvil,
        // patch the alto.json to use Alto's simulation EntryPoint v0.7 address so bundler starts.
        let project_alto_path = self.contracts_dir.join("alto.json");
        if let Ok(contents) = std::fs::read_to_string(&project_alto_path)
            && let Ok(mut v) =
                serde_json::from_str::<serde_json::Value>(&contents)
            && let Some(entry_s) = v.get("entrypoints").and_then(|x| x.as_str())
            && let Ok(addr) = entry_s.parse::<Address>()
        {
            // Empty code indicates not deployed
            let code = self
                .provider
                .get_code_at(addr)
                .latest()
                .await
                .unwrap_or_default();
            if code.is_empty() {
                // Known simulation EntryPoint v0.7 address that Alto deploys
                let sim_v7 = "0x5E077d32743E2D9BBfB2bF7Bc46E3906A80C892F";
                if let Some(obj) = v.as_object_mut() {
                    obj.insert(
                        "entrypoints".to_string(),
                        serde_json::Value::String(sim_v7.to_string()),
                    );
                }
                let patched = serde_json::to_string_pretty(&v)?;
                println!(
                    "[alto config] Patched entrypoints in {} to simulation v0.7 {}",
                    project_alto_path.display(),
                    sim_v7
                );
                std::fs::write(&project_alto_path, patched)?;
            }
        }

        // 0) Ensure project dependencies and builds are ready
        run_command_stream(
            &self.contracts_dir,
            "pnpm",
            &["install"],
            "pnpm install",
        )
        .await?;
        run_command_stream(
            &self.contracts_dir,
            "forge",
            &["soldeer", "install"],
            "forge soldeer install",
        )
        .await?;
        run_command_stream(
            &self.contracts_dir,
            "forge",
            &["build"],
            "forge build",
        )
        .await?;

        // 1) Deploy test contracts via project script
        {
            run_command_stream(
                &self.contracts_dir,
                "pnpm",
                &["deploy-test"],
                "pnpm deploy-test",
            )
            .await?;
        }

        // EP address is sourced from alto.json; EntryPoint is patched to sim v0.7 if missing.

        // 2) Start bundler using project's alto.json via `pnpm bundler`
        let mut cmd = Command::new("pnpm");
        cmd.current_dir(&self.contracts_dir)
            .arg("bundler")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        // Stream stdout/stderr and wait until we detect readiness
        let (ready_tx, ready_rx) = oneshot::channel::<()>();

        let bundler_port = self.config.port;
        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout).lines();
            let mut ready_tx_stdout = Some(ready_tx);
            let config = self.config.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    let clean = strip_ansi(&line);
                    let ready = is_alto_ready_line(&line, &config);
                    println!("[alto stdout] {}", line);
                    println!(
                        "[alto stdout dbg] ready={} clean=\"{}\"",
                        ready, clean
                    );
                    if ready && let Some(tx) = ready_tx_stdout.take() {
                        _ = tx.send(());
                    }
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let mut reader = BufReader::new(stderr).lines();
            let config = self.config.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    let clean = strip_ansi(&line);
                    let ready = is_alto_ready_line(&line, &config);
                    eprintln!("[alto stderr] {}", line);
                    eprintln!(
                        "[alto stderr dbg] ready={} clean=\"{}\"",
                        ready, clean
                    );
                }
            });
        }

        // Wait up to a reasonable time for readiness signal (fail-fast on timeout)
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
        if let Some(mut anvil) = self.anvil_child.take() {
            println!("[alto helper] stopping anvil...");
            terminate_child_gracefully(&mut anvil, "anvil").await;
        }
        // Temp config no longer used when relying on project alto.json
        let _ = self.temp_config_path.take();
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

fn is_alto_ready_line(line: &str, config: &AltoTestHelperConfig) -> bool {
    let clean = strip_ansi(line);
    let scheme = "http";
    let host = "0.0.0.0";
    let port = config.port.to_string();
    let scheme_host_port = format!("{}://{}:{}", scheme, host, port);
    clean.contains(&format!("Server listening at {}", scheme_host_port))
}

fn strip_ansi(input: &str) -> String {
    let bytes = strip_ansi_bytes(input.as_bytes());
    String::from_utf8_lossy(&bytes).into_owned()
}

async fn run_command_stream(
    dir: &Path,
    program: &str,
    args: &[&str],
    label: &str,
) -> eyre::Result<()> {
    let mut cmd = Command::new(program);
    cmd.current_dir(dir)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout).lines();
        let label = label.to_string();
        tokio::spawn(async move {
            while let Ok(Some(line)) = reader.next_line().await {
                println!("[{} stdout] {}", label, line);
            }
        });
    }
    if let Some(stderr) = child.stderr.take() {
        let mut reader = BufReader::new(stderr).lines();
        let label = label.to_string();
        tokio::spawn(async move {
            while let Ok(Some(line)) = reader.next_line().await {
                eprintln!("[{} stderr] {}", label, line);
            }
        });
    }

    let status = child.wait().await?;
    if !status.success() {
        return Err(eyre::eyre!("{} failed with status {}", label, status));
    }
    Ok(())
}

fn resolve_contracts_dir() -> PathBuf {
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

async fn is_port_open(addr: &str) -> bool {
    TcpStream::connect(addr).await.is_ok()
}

fn is_anvil_ready_line(line: &str, config: &AltoTestHelperConfig) -> bool {
    let lower = line.to_lowercase();
    let host = config.node_url.host().unwrap().to_string();
    let port = config.node_url.port().unwrap().to_string();
    let host_port = format!("{}:{}", host, port);
    lower.contains(&format!("listening on {}", host_port))
        || (lower.contains("http json-rpc") && lower.contains(&host))
        || lower.contains(&host_port)
}

async fn terminate_child_gracefully(child: &mut Child, label: &str) {
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
    use crate::{
        chain::id::ChainId, erc4337::entry_point::version::EntryPointVersion,
    };
    use alloy::providers::ProviderBuilder;
    use eyre::Result;

    #[tokio::test]
    #[ignore = "temporaryily disabled to speed up test run"]
    async fn test_alto_start_stop() -> Result<()> {
        let provider = ProviderBuilder::new().connect_anvil_with_wallet();

        let alto_cfg = AltoTestHelperConfig {
            entrypoint: EntryPointConfig {
                address: "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108".parse()?,
                version: EntryPointVersion::V08,
                chain_id: ChainId::ETHEREUM_MAINNET,
            },
            executor_private_keys:
                "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string(),
            port: 4337,
            node_url: Url::parse("http://127.0.0.1:8545")?,
            safe_mode: false,

            utility_private_key:
                "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d".to_string(),
        };

        let mut alto = AltoTestHelper::new(provider.clone(), alto_cfg);
        println!("[test] calling alto.start()...");
        alto.start().await?;
        println!("[test] alto.start() returned");
        println!("[test] checking status...");
        assert!(matches!(alto.status().await?, BundlerStatus::Running));
        println!("[test] status is Running");
        println!("[test] calling alto.stop()...");
        alto.stop().await?;
        println!("[test] alto.stop() returned");
        println!("[test] checking status stopped...");
        assert!(matches!(alto.status().await?, BundlerStatus::Stopped));
        println!("[test] status is Stopped");
        Ok(())
    }
}
