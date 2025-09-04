use crate::config;
use sdk::api::{
    account::{
        modular_account::{
            CredentialDetails as SdkCredentialDetails,
            DeployModularAccountArgs as SdkDeployModularAccountArgs,
            DeployedModularAccountDetails as SdkDeployedModularAccountDetails,
            PasskeyModuleArgs as SdkPasskeyModuleArgs,
            SessionModuleArgs as SdkSessionModuleArgs,
            deploy_modular_account as sdk_deploy_modular_account,
        },
        session::decode_session_config,
    },
    utils::{parse_address, parse_paymaster_params},
};

#[derive(Debug, uniffi::Record)]
pub struct CredentialDetails {
    pub id: String,
    pub public_key: Vec<u8>,
}

impl From<CredentialDetails> for SdkCredentialDetails {
    fn from(credential: CredentialDetails) -> Self {
        SdkCredentialDetails {
            id: credential.id,
            public_key: credential.public_key,
        }
    }
}

#[derive(Debug, uniffi::Record)]
pub struct SessionModuleArgs {
    pub location: String,
    pub initial_session_config_json: Option<String>,
}

impl From<SessionModuleArgs> for SdkSessionModuleArgs {
    fn from(session_module: SessionModuleArgs) -> Self {
        let location = parse_address(&session_module.location)
            .expect("Invalid session module location address");

        let initial_session =
            session_module.initial_session_config_json.map(|config_json| {
                decode_session_config(&config_json)
                    .expect("Invalid session config JSON")
            });

        SdkSessionModuleArgs { location, initial_session }
    }
}

#[derive(Debug, uniffi::Record)]
pub struct PasskeyModuleArgs {
    pub location: String,
    pub credential: CredentialDetails,
    pub expected_origin: Option<String>,
}

impl From<PasskeyModuleArgs> for SdkPasskeyModuleArgs {
    fn from(passkey_module: PasskeyModuleArgs) -> Self {
        let location = parse_address(&passkey_module.location)
            .expect("Invalid passkey module location address");

        SdkPasskeyModuleArgs {
            location,
            credential: passkey_module.credential.into(),
            expected_origin: passkey_module.expected_origin,
        }
    }
}

#[derive(Debug, uniffi::Record)]
pub struct PaymasterParams {
    pub paymaster_address: String,
    pub paymaster_input: Option<String>,
}

#[derive(Debug, uniffi::Record)]
pub struct DeployModularAccountArgs {
    pub install_no_data_modules: Vec<String>,
    pub owners: Vec<String>,
    pub session_module: Option<SessionModuleArgs>,
    pub paymaster: Option<PaymasterParams>,
    pub passkey_module: Option<PasskeyModuleArgs>,
    pub unique_account_id: Option<String>,
}

impl TryInto<SdkDeployModularAccountArgs> for DeployModularAccountArgs {
    type Error = DeployModularAccountError;

    fn try_into(self) -> Result<SdkDeployModularAccountArgs, Self::Error> {
        let args = self;

        let install_no_data_modules = args
            .install_no_data_modules
            .into_iter()
            .map(|addr| parse_address(&addr))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                DeployModularAccountError::InvalidModuleAddress(e.to_string())
            })?;

        let owners = args
            .owners
            .into_iter()
            .map(|owner| parse_address(&owner))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                DeployModularAccountError::InvalidOwnerAddress(e.to_string())
            })?;

        let session_module = if let Some(session_module) = args.session_module {
            let location =
                parse_address(&session_module.location).map_err(|e| {
                    DeployModularAccountError::InvalidSessionModuleLocation(
                        e.to_string(),
                    )
                })?;

            let initial_session = session_module
                .initial_session_config_json
                .map(|config_json| {
                    decode_session_config(&config_json).map_err(|e| {
                        DeployModularAccountError::InvalidSessionConfig(
                            e.to_string(),
                        )
                    })
                })
                .transpose()?;

            Some(SdkSessionModuleArgs { location, initial_session })
        } else {
            None
        };

        let passkey_module = if let Some(passkey_module) = args.passkey_module {
            let location =
                parse_address(&passkey_module.location).map_err(|e| {
                    DeployModularAccountError::InvalidPasskeyModuleLocation(
                        e.to_string(),
                    )
                })?;

            Some(SdkPasskeyModuleArgs {
                location,
                credential: passkey_module.credential.into(),
                expected_origin: passkey_module.expected_origin,
            })
        } else {
            None
        };

        let paymaster = args
            .paymaster
            .map(|p| {
                parse_paymaster_params(p.paymaster_address, p.paymaster_input)
                    .map_err(|e| {
                        DeployModularAccountError::InvalidPaymasterParams(
                            e.to_string(),
                        )
                    })
            })
            .transpose()?;

        Ok(SdkDeployModularAccountArgs {
            install_no_data_modules,
            owners,
            session_module,
            paymaster,
            passkey_module,
            unique_account_id: args.unique_account_id,
        })
    }
}

#[derive(Debug, uniffi::Record)]
pub struct DeployedModularAccountDetails {
    pub address: String,
    pub unique_account_id: String,
    pub transaction_receipt_json: String,
}

impl From<SdkDeployedModularAccountDetails> for DeployedModularAccountDetails {
    fn from(details: SdkDeployedModularAccountDetails) -> Self {
        Self {
            address: details.address.to_string(),
            unique_account_id: details.unique_account_id,
            transaction_receipt_json: details.transaction_receipt_json,
        }
    }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum DeployModularAccountError {
    #[error("{0}")]
    Deploy(String),

    #[error("Invalid module address: {0}")]
    InvalidModuleAddress(String),

    #[error("Invalid owner address: {0}")]
    InvalidOwnerAddress(String),

    #[error("Invalid session module location: {0}")]
    InvalidSessionModuleLocation(String),

    #[error("Invalid passkey module location: {0}")]
    InvalidPasskeyModuleLocation(String),

    #[error("Invalid session config: {0}")]
    InvalidSessionConfig(String),

    #[error("Invalid paymaster params: {0}")]
    InvalidPaymasterParams(String),
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn deploy_modular_account(
    args: DeployModularAccountArgs,
    config: config::Config,
) -> Result<DeployedModularAccountDetails, DeployModularAccountError> {
    let sdk_config = config.try_into().map_err(|e: config::ConfigError| {
        DeployModularAccountError::Deploy(e.to_string())
    })?;

    let sdk_args = args.try_into()?;

    sdk_deploy_modular_account(sdk_args, &sdk_config)
        .await
        .map_err(|e| DeployModularAccountError::Deploy(e.to_string()))
        .map(Into::into)
}
