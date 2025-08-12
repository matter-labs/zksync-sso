use sdk::config::{
    self, Config as SdkConfig, deploy_wallet::DeployWallet as SdkDeployWallet,
};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ConfigError {
    #[error("Invalid contract address: {0}")]
    InvalidContractAddress(String),
    #[error("Invalid deploy wallet: {0}")]
    InvalidDeployWallet(String),
    #[error("Invalid node URL: {0}")]
    InvalidNodeUrl(String),
    #[error("Failed to write config file: {0}")]
    WriteError(String),
}

#[derive(Debug, uniffi::Record)]
pub struct SSOContracts {
    pub account_factory: String,
    pub passkey: String,
    pub session: String,
    pub account_paymaster: String,
    pub recovery: String,
}

#[derive(Debug, uniffi::Record)]
pub struct DeployWallet {
    pub private_key_hex: String,
}

#[derive(Debug, uniffi::Record)]
pub struct Config {
    pub contracts: SSOContracts,
    pub node_url: String,
    pub deploy_wallet: Option<DeployWallet>,
}

impl TryFrom<Config> for SdkConfig {
    type Error = ConfigError;

    fn try_from(config: Config) -> Result<Self, ConfigError> {
        SdkConfig::with_url_str(
            config::contracts::SSOContracts::with_address_strs(
                &config.contracts.account_factory,
                &config.contracts.passkey,
                &config.contracts.session,
                &config.contracts.account_paymaster,
                &config.contracts.recovery,
            )
            .map_err(|e| ConfigError::InvalidContractAddress(e.to_string()))?,
            &config.node_url,
            config
                .deploy_wallet
                .map(|deploy_wallet| {
                    SdkDeployWallet::try_from(deploy_wallet.private_key_hex)
                        .map_err(|e| {
                            ConfigError::InvalidDeployWallet(e.to_string())
                        })
                })
                .transpose()?,
        )
        .map_err(|e| ConfigError::InvalidNodeUrl(e.to_string()))
    }
}

impl From<SdkConfig> for Config {
    fn from(sdk_config: SdkConfig) -> Self {
        Self {
            contracts: SSOContracts {
                account_factory: sdk_config
                    .contracts
                    .account_factory
                    .to_string(),
                passkey: sdk_config.contracts.passkey.to_string(),
                session: sdk_config.contracts.session.to_string(),
                account_paymaster: sdk_config
                    .contracts
                    .account_paymaster
                    .to_string(),
                recovery: sdk_config.contracts.recovery.to_string(),
            },
            node_url: sdk_config.node_url.to_string(),
            deploy_wallet: sdk_config.deploy_wallet.map(|dw| DeployWallet {
                private_key_hex: dw.private_key_hex.to_string(),
            }),
        }
    }
}
