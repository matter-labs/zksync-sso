use crate::{
    client::modular_account::{
        DeployModularAccountArgs as ClientDeployModularAccountArgs,
        PasskeyModuleArgs as ClientPasskeyModuleArgs,
        SessionModuleArgs as ClientSessionModuleArgs,
        deploy_modular_account as client_deploy_modular_account,
    },
    config::Config,
    utils::session::session_lib::session_spec::SessionSpec,
};
use alloy::primitives::Address;
use alloy_zksync::network::unsigned_tx::eip712::PaymasterParams;

#[derive(Debug, Clone)]
pub struct CredentialDetails {
    pub id: String,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SessionModuleArgs {
    pub location: Address,
    pub initial_session: Option<SessionSpec>,
}

#[derive(Debug, Clone)]
pub struct PasskeyModuleArgs {
    pub location: Address,
    pub credential: CredentialDetails,
    pub expected_origin: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeployModularAccountArgs {
    pub install_no_data_modules: Vec<Address>,
    pub owners: Vec<Address>,
    pub session_module: Option<SessionModuleArgs>,
    pub paymaster: Option<PaymasterParams>,
    pub passkey_module: Option<PasskeyModuleArgs>,
    pub unique_account_id: Option<String>,
}

pub struct DeployedModularAccountDetails {
    pub address: Address,
    pub unique_account_id: String,
    pub transaction_receipt_json: String,
}

pub async fn deploy_modular_account(
    args: DeployModularAccountArgs,
    config: &Config,
) -> eyre::Result<DeployedModularAccountDetails> {
    let session_module =
        args.session_module.map(|session| ClientSessionModuleArgs {
            location: session.location,
            initial_session: session.initial_session,
        });

    let passkey_module =
        args.passkey_module.map(|passkey| ClientPasskeyModuleArgs {
            location: passkey.location,
            credential:
                crate::client::passkey::actions::deploy::CredentialDetails {
                    id: passkey.credential.id,
                    public_key: passkey.credential.public_key,
                },
            expected_origin: passkey.expected_origin,
        });

    let client_args = ClientDeployModularAccountArgs {
        install_no_data_modules: args.install_no_data_modules,
        owners: args.owners,
        session_module,
        paymaster: args.paymaster,
        passkey_module,
        unique_account_id: args.unique_account_id,
    };

    let deployed_account_details =
        client_deploy_modular_account(client_args, config).await?;

    let address = deployed_account_details.address;
    let unique_account_id =
        alloy::hex::encode(deployed_account_details.unique_account_id);
    let transaction_receipt_json =
        serde_json::to_string(&deployed_account_details.transaction_receipt)
            .map_err(|e| {
                eyre::eyre!("Failed to serialize transaction receipt: {}", e)
            })?;

    Ok(DeployedModularAccountDetails {
        address,
        unique_account_id,
        transaction_receipt_json,
    })
}
