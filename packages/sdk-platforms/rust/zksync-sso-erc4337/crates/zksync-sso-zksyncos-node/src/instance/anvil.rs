use eyre::{Result, WrapErr};
use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
};
const ANVIL_PORT: u16 = 8545;

pub(crate) fn spawn_l1_anvil(
    repo_dir: &Path,
    logs_dir: &Path,
    print_logs: bool,
) -> Result<Child> {
    let state_path = repo_dir.join("zkos-l1-state.json");
    eyre::ensure!(
        state_path.exists(),
        "zksync-os-server checkout missing zkos-l1-state.json"
    );

    let mut cmd = Command::new("anvil");
    cmd.arg("--port").arg(ANVIL_PORT.to_string()).arg("--load-state").arg(
        state_path.to_str().ok_or_else(|| eyre::eyre!("invalid state path"))?,
    );

    if print_logs {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    } else {
        let stdout = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(logs_dir.join("anvil.log"))
            .wrap_err("failed to open anvil log file")?;
        let stderr = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(logs_dir.join("anvil.err.log"))
            .wrap_err("failed to open anvil err log file")?;
        cmd.stdout(Stdio::from(stdout)).stderr(Stdio::from(stderr));
    }

    let child =
        cmd.spawn().wrap_err("failed to spawn anvil for zksync-os-server")?;
    Ok(child)
}
