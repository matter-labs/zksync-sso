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
    state_version: &str,
) -> Result<Child> {
    let state_path = repo_dir
        .join(format!("./local-chains/{}/default/", state_version))
        .join("zkos-l1-state.json");
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

#[cfg(test)]
mod tests {
    use super::*;
    use eyre::WrapErr;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::{fs, io::Write};
    use tempfile::tempdir;

    #[cfg(unix)]
    struct EnvGuard {
        key: &'static str,
        old: String,
    }

    #[cfg(unix)]
    impl EnvGuard {
        fn set(key: &'static str, value: String) -> Self {
            let old = std::env::var(key).unwrap_or_default();
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, old }
        }
    }

    #[cfg(unix)]
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                std::env::set_var(self.key, &self.old);
            }
        }
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "manual test"]
    fn spawn_l1_anvil_runs() -> Result<()> {
        let repo_dir = tempdir()?;
        let logs_dir = tempdir()?;

        let state_dir = repo_dir.path().join("local-chains/v31.0/default");
        fs::create_dir_all(&state_dir)?;
        fs::write(state_dir.join("zkos-l1-state.json"), "{}")?;

        let bin_dir = tempdir()?;
        let anvil_path = bin_dir.path().join("anvil");
        let mut file = fs::File::create(&anvil_path)?;
        writeln!(file, "#!/bin/sh\nsleep 2\n")?;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&anvil_path, perms)?;

        let new_path = format!(
            "{}:{}",
            bin_dir.path().display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let _path_guard = EnvGuard::set("PATH", new_path);

        let mut child =
            spawn_l1_anvil(repo_dir.path(), logs_dir.path(), false, "v31.0")?;

        let status = child.kill();
        let _ = child.wait();
        status.wrap_err("failed to kill anvil child")?;
        Ok(())
    }
}
