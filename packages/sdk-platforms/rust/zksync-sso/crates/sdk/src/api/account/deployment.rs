use crate::{
    api::account::{
        modular_account::{
            CredentialDetails as ModularCredentialDetails,
            DeployModularAccountArgs,
            PasskeyModuleArgs as ModularPasskeyModuleArgs, SessionModuleArgs,
            deploy_modular_account as client_deploy_modular_account,
        },
        passkey::passkey_parameters::{
            PasskeyParameters, parse_passkey_parameters,
        },
    },
    client::passkey::{
        account_factory::{AccountParams, create_account},
        actions::deploy::{
            CredentialDetails, DeployAccountArgs,
            deploy_account as client_deploy_account,
        },
    },
    config::Config,
    utils::session::session_lib::session_spec::SessionSpec,
};
use alloy::primitives::{Address, Bytes};
use alloy_zksync::network::unsigned_tx::eip712::PaymasterParams;
use log::debug;

pub struct DeployedAccountDetails {
    pub address: Address,
    pub unique_account_id: String,
    pub transaction_receipt_json: Option<String>,
}

pub async fn deploy_account(
    passkey_parameters: PasskeyParameters,
    initial_k1_owners: Option<Vec<Address>>,
    initial_session: Option<SessionSpec>,
    config: &Config,
) -> eyre::Result<DeployedAccountDetails> {
    debug!("XDB deploy_account - passkey_parameters: {passkey_parameters:?}");

    let parsed_params = parse_passkey_parameters(&passkey_parameters).await?;

    debug!("XDB deploy_account - parsed_params: {parsed_params:?}");

    let paymaster = Some(PaymasterParams {
        paymaster: config.contracts.account_paymaster,
        paymaster_input: Bytes::new(),
    });

    let deploy_account_args = DeployAccountArgs {
        credential: CredentialDetails {
            id: parsed_params.credential.id,
            public_key: parsed_params.credential.public_key,
        },
        expected_origin: Some(parsed_params.expected_origin),
        contracts: config.contracts,
        paymaster,
        initial_k1_owners,
        initial_session,
        ..Default::default()
    };

    let deployed_account_details =
        client_deploy_account(deploy_account_args, config).await?;

    let address = deployed_account_details.address;
    let unique_account_id =
        alloy::hex::encode(deployed_account_details.unique_account_id);
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
    initial_k1_owners: Option<Vec<Address>>,
    initial_session: Option<SessionSpec>,
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
        initial_k1_owners,
        initial_session,
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

pub async fn deploy_modular_account(
    passkey_parameters: Option<PasskeyParameters>,
    initial_k1_owners: Option<Vec<Address>>,
    initial_session: Option<SessionSpec>,
    config: &Config,
) -> eyre::Result<DeployedAccountDetails> {
    debug!(
        "XDB deploy_modular_account_with_passkey - passkey_parameters: {passkey_parameters:?}"
    );

    let paymaster = Some(PaymasterParams {
        paymaster: config.contracts.account_paymaster,
        paymaster_input: Bytes::new(),
    });

    // Map parsed parameters to PasskeyModuleArgs if provided
    let passkey_module = if let Some(passkey_params) = passkey_parameters {
        let parsed_params = parse_passkey_parameters(&passkey_params).await?;

        debug!(
            "XDB deploy_modular_account_with_passkey - parsed_params: {parsed_params:?}"
        );

        Some(ModularPasskeyModuleArgs {
            location: config.contracts.passkey,
            credential: ModularCredentialDetails {
                id: parsed_params.credential.id,
                public_key: parsed_params.credential.public_key,
            },
            expected_origin: Some(parsed_params.expected_origin),
        })
    } else {
        None
    };

    let session_module = initial_session.map(|session| SessionModuleArgs {
        location: config.contracts.session,
        initial_session: Some(session),
    });

    let deploy_modular_account_args = DeployModularAccountArgs {
        install_no_data_modules: vec![],
        owners: initial_k1_owners.unwrap_or_default(),
        session_module,
        paymaster,
        passkey_module,
        unique_account_id: None,
    };

    let deployed_account_details =
        client_deploy_modular_account(deploy_modular_account_args, config)
            .await?;

    Ok(DeployedAccountDetails {
        address: deployed_account_details.address,
        unique_account_id: deployed_account_details.unique_account_id,
        transaction_receipt_json: Some(
            deployed_account_details.transaction_receipt_json,
        ),
    })
}
