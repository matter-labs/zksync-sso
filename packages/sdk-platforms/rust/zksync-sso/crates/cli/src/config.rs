use eyre::Result;
use sdk::config::Config;
use std::{fs, path::PathBuf};

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load() -> Result<Config> {
        let config_path = Self::get_config_path()?;
        Self::load_from_path(&config_path)
    }

    pub fn load_from_path(path: &PathBuf) -> Result<Config> {
        let config_json = fs::read_to_string(path).map_err(|e| {
            eyre::eyre!("Failed to read config file at {:?}: {}", path, e)
        })?;

        serde_json::from_str(&config_json)
            .map_err(|e| eyre::eyre!("Failed to parse config JSON: {}", e))
    }

    pub fn get_config_path() -> Result<PathBuf> {
        // First try manifest dir (for development and tests)
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let dev_config_path =
                PathBuf::from(&manifest_dir).join("config.json");
            if dev_config_path.exists() {
                return Ok(dev_config_path);
            }

            // Try workspace root config for tests
            // CLI crate is at: /path/to/workspace/crates/cli
            // Config is at: /path/to/workspace/target/debug/config.json
            let manifest_path = PathBuf::from(&manifest_dir);
            if let Some(workspace_root) =
                manifest_path.parent().and_then(|p| p.parent())
            {
                let workspace_config =
                    workspace_root.join("target/debug/config.json");
                if workspace_config.exists() {
                    return Ok(workspace_config);
                }
            }
        }

        // Then try executable directory (for production)
        if let Ok(exe_path) = std::env::current_exe()
            && let Some(exe_dir) = exe_path.parent()
        {
            let config_path = exe_dir.join("config.json");
            if config_path.exists() {
                return Ok(config_path);
            }
        }

        Err(eyre::eyre!(
            "Config file not found. Please run 'deploy-contracts' command first to generate the config."
        ))
    }

    pub fn get_cli_config_write_path() -> PathBuf {
        if let Ok(exe_path) = std::env::current_exe()
            && let Some(exe_dir) = exe_path.parent()
        {
            return exe_dir.join("config.json");
        }

        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            return PathBuf::from(manifest_dir).join("config.json");
        }

        PathBuf::from("./config.json")
    }

    pub fn get_swift_config_path() -> PathBuf {
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            // CARGO_MANIFEST_DIR is: .../zksync-sso/packages/sdk-platforms/rust/zksync-sso/crates/cli
            // We need to go up 6 levels to reach the workspace root
            let workspace_root = PathBuf::from(manifest_dir)
                .parent() // -> .../crates
                .and_then(|p| p.parent()) // -> .../zksync-sso (rust)
                .and_then(|p| p.parent()) // -> .../rust
                .and_then(|p| p.parent()) // -> .../sdk-platforms
                .and_then(|p| p.parent()) // -> .../packages
                .and_then(|p| p.parent()) // -> .../zksync-sso (workspace root)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));

            return workspace_root.join("packages/sdk-platforms/swift/ZKsyncSSO/Sources/ZKsyncSSO/Config/config.json");
        }

        PathBuf::from(
            "../../swift/ZKsyncSSO/Sources/ZKsyncSSO/Config/config.json",
        )
    }

    pub fn get_react_native_config_path() -> PathBuf {
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            // CARGO_MANIFEST_DIR is: .../zksync-sso/packages/sdk-platforms/rust/zksync-sso/crates/cli
            // We need to go up 6 levels to reach the workspace root
            let workspace_root = PathBuf::from(manifest_dir)
                .parent() // -> .../crates
                .and_then(|p| p.parent()) // -> .../zksync-sso (rust)
                .and_then(|p| p.parent()) // -> .../rust
                .and_then(|p| p.parent()) // -> .../sdk-platforms
                .and_then(|p| p.parent()) // -> .../packages
                .and_then(|p| p.parent()) // -> .../zksync-sso (workspace root)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));

            return workspace_root.join("packages/sdk-platforms/react-native/react-native-zksync-sso/example/src/config.json");
        }

        PathBuf::from(
            "../../react-native/react-native-zksync-sso/example/src/config.json",
        )
    }

    pub fn get_all_default_config_paths() -> Vec<PathBuf> {
        vec![
            Self::get_swift_config_path(),
            Self::get_react_native_config_path(),
            Self::get_cli_config_write_path(),
        ]
    }
}
