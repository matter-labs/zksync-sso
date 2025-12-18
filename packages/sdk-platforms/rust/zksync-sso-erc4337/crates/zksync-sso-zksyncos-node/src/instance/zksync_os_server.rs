use eyre::{Result, WrapErr};
use serde_json::json;
use std::{
    ffi::OsStr,
    fs,
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use url::Url;

pub const DEFAULT_REPO_URL: &str =
    "https://github.com/matter-labs/zksync-os-server";
pub const DEFAULT_L1_RPC_URL: &str = "http://127.0.0.1:8545";
pub const DEFAULT_L2_RPC_URL: &str = "http://127.0.0.1:3050";
const SERVER_BIN_NAME: &str = "zksync-os-server";
const READY_TIMEOUT: Duration = Duration::from_secs(90);

pub(crate) fn spawn_server_process(
    repo_dir: &Path,
    logs_dir: &Path,
    print_logs: bool,
    config_path: &Path,
) -> Result<Child> {
    let binary_path =
        repo_dir.join("target").join("release").join(SERVER_BIN_NAME);
    eyre::ensure!(
        binary_path.exists(),
        "zksync-os-server binary missing after build"
    );
    eyre::ensure!(
        config_path.exists(),
        "zksync-os-server config missing at {}",
        config_path.display()
    );

    let mut cmd = Command::new(binary_path);
    cmd.current_dir(repo_dir);
    cmd.arg("--config").arg(config_path);

    if print_logs {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    } else {
        let log_file_path = logs_dir.join("server.log");
        let stdout = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
            .wrap_err("failed to open zksync-os-server log file")?;
        let stderr = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(logs_dir.join("server.err.log"))
            .wrap_err("failed to open zksync-os-server err log file")?;
        cmd.stdout(Stdio::from(stdout)).stderr(Stdio::from(stderr));
    }

    let child =
        cmd.spawn().wrap_err("failed to launch zksync-os-server process")?;
    Ok(child)
}

pub(crate) fn wait_for_server(rpc_url: &Url) -> Result<()> {
    let agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(2))
        .timeout_write(Duration::from_secs(2))
        .build();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    })
    .to_string();

    let deadline = Instant::now() + READY_TIMEOUT;
    let endpoint = rpc_url.as_str().to_string();

    loop {
        match agent
            .post(&endpoint)
            .set("Content-Type", "application/json")
            .send_string(&body)
        {
            Ok(resp) if resp.status() == 200 => return Ok(()),
            Ok(_) | Err(_) => {
                if Instant::now() > deadline {
                    eyre::bail!(
                        "zksync-os-server did not respond on {} within {:?}",
                        endpoint,
                        READY_TIMEOUT
                    );
                }
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}

pub(crate) fn ensure_binary_exists(path: &Path) -> Result<()> {
    let binary_path = path.join("target").join("release").join(SERVER_BIN_NAME);
    eyre::ensure!(
        binary_path.exists(),
        "zksync-os-server binary missing in {}",
        path.display()
    );
    Ok(())
}

pub(crate) fn build_repo(path: &Path) -> Result<()> {
    let status = Command::new("cargo")
        .args(["build", "--release", "--bin", SERVER_BIN_NAME])
        .current_dir(path)
        .status()
        .wrap_err("failed to build zksync-os-server")?;

    eyre::ensure!(status.success(), "cargo build exited with {:?}", status);
    ensure_binary_exists(path)
}

pub(crate) fn ensure_repo(path: &Path, repo_url: &str) -> Result<()> {
    if path.join("Cargo.toml").exists() {
        return Ok(());
    }

    let status = Command::new("git")
        .args([
            OsStr::new("clone"),
            OsStr::new("--depth"),
            OsStr::new("1"),
            OsStr::new(repo_url),
            path.as_os_str(),
        ])
        .status()
        .wrap_err("failed to clone zksync-os-server repository")?;

    eyre::ensure!(status.success(), "git clone exited with {:?}", status);
    Ok(())
}
