use crate::{
    client::passkey::{
        account_factory::{AccountParams, create_account},
        actions::deploy::CredentialDetails,
    },
    config::Config,
};
use alloy::primitives::{Address, Bytes};
use alloy_zksync::network::unsigned_tx::eip712::PaymasterParams;
use log::debug;
use parse_passkey_parameters::parse_passkey_parameters;

pub mod parse_passkey_parameters;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidRpId {
    pub origin: String,
    pub rp_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpId {
    Apple(String),
    Android(AndroidRpId),
}

impl RpId {
    pub fn origin(&self) -> String {
        match self {
            RpId::Apple(rp_id) => format!("https://{}", rp_id),
            RpId::Android(android_rp_id) => android_rp_id.origin.to_string(),
        }
    }

    pub fn rp_id(&self) -> String {
        match self {
            RpId::Apple(rp_id) => rp_id.to_string(),
            RpId::Android(android_rp_id) => android_rp_id.rp_id.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PasskeyParameters {
    pub credential_raw_attestation_object: Vec<u8>,
    pub credential_raw_client_data_json: Vec<u8>,
    pub credential_id: Vec<u8>,
    pub rp_id: RpId,
}

pub struct DeployedAccountDetails {
    pub address: Address,
    pub unique_account_id: String,
    pub transaction_receipt_json: Option<String>,
}

pub async fn deploy_account(
    passkey_parameters: PasskeyParameters,
    config: &Config,
) -> eyre::Result<DeployedAccountDetails> {
    debug!("XDB deploy_account - passkey_parameters: {:?}", passkey_parameters);

    let parsed_params = parse_passkey_parameters(&passkey_parameters).await?;

    debug!("XDB deploy_account - parsed_params: {:?}", parsed_params);

    let paymaster = Some(PaymasterParams {
        paymaster: config.contracts.account_paymaster,
        paymaster_input: Bytes::new(),
    });

    let deploy_account_args =
        crate::client::passkey::actions::deploy::DeployAccountArgs {
            credential: CredentialDetails {
                id: parsed_params.credential.id,
                public_key: parsed_params.credential.public_key,
            },
            expected_origin: Some(parsed_params.expected_origin),
            contracts: config.contracts.clone(),
            paymaster,
            ..Default::default()
        };

    let deployed_account_details =
        crate::client::passkey::actions::deploy::deploy_account(
            deploy_account_args,
            config,
        )
        .await?;

    let address = deployed_account_details.address;
    let unique_account_id =
        hex::encode(deployed_account_details.unique_account_id);
    let transaction_receipt_json =
        serde_json::to_string(&deployed_account_details.transaction_receipt)
            .map_err(|e| {
                eyre::eyre!("Failed to serialize transaction receipt: {}", e)
            })?;

    Ok(DeployedAccountDetails {
        address,
        unique_account_id,
        transaction_receipt_json: Some(transaction_receipt_json),
    })
}

pub async fn deploy_account_with_unique_id(
    passkey_parameters: PasskeyParameters,
    unique_account_id: String,
    config: &Config,
) -> eyre::Result<DeployedAccountDetails> {
    let parsed_params = parse_passkey_parameters(&passkey_parameters).await?;

    let paymaster = Some(PaymasterParams {
        paymaster: config.contracts.account_paymaster,
        paymaster_input: Bytes::new(),
    });

    let credential = CredentialDetails {
        id: parsed_params.credential.id,
        public_key: parsed_params.credential.public_key,
    };

    let address = create_account(
        unique_account_id.clone(),
        credential,
        &AccountParams {
            passkey_expected_origin: parsed_params.expected_origin,
        },
        paymaster,
        config,
    )
    .await?;

    Ok(DeployedAccountDetails {
        address,
        unique_account_id,
        transaction_receipt_json: None,
    })
}
