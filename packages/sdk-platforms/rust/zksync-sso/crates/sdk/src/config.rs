pub mod contracts;

use crate::config::contracts::PasskeyContracts;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{env, fs, io::Write, path::PathBuf};
use url::Url;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub contracts: PasskeyContracts,
    pub node_url: Url,
}

impl Config {
    pub fn new(contracts: PasskeyContracts, node_url: Url) -> Self {
        Self { contracts, node_url }
    }

    pub fn with_url_str(
        contracts: PasskeyContracts,
        node_url: &str,
    ) -> Result<Self> {
        Ok(Self {
            contracts,
            node_url: node_url
                .parse()
                .map_err(|e| eyre::eyre!("Invalid node URL: {}", e))?,
        })
    }

    pub fn write_json(&self, config_path: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(&self)
            .map_err(|e| eyre::eyre!("Failed to serialize config: {}", e))?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                eyre::eyre!("Failed to create config directory: {}", e)
            })?;
        }

        let mut file = fs::File::create(config_path)
            .map_err(|e| eyre::eyre!("Failed to create config file: {}", e))?;

        file.write_all(json.as_bytes())
            .map_err(|e| eyre::eyre!("Failed to write config file: {}", e))?;

        println!("Wrote config to: {:?}", config_path);
        Ok(())
    }

    pub fn get_default_swift_config_path() -> PathBuf {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let workspace_root = PathBuf::from(manifest_dir);
        workspace_root.join(
            "../../../../swift/ZKsyncSSO/Sources/ZKsyncSSO/Config/config.json",
        )
    }

    pub fn local() -> Self {
        let config_path = Self::get_default_swift_config_path();
        let config_json = fs::read_to_string(&config_path)
            .expect("Failed to read config file");

        serde_json::from_str(&config_json).expect("Failed to parse config JSON")
    }
}
