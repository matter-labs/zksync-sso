use crate::{
    client::passkey::{
        account_factory::{AccountParams, create_account},
        actions::deploy::CredentialDetails,
    },
    config::Config,
    utils::passkey::passkey::apple::{
        extract_public_key, verify::verify_registration,
    },
};
use alloy::primitives::{Address, Bytes};
use alloy_zksync::network::unsigned_tx::eip712::PaymasterParams;

pub struct PasskeyParameters {
    pub credential_raw_attestation_object: Vec<u8>,
    pub credential_raw_client_data_json: Vec<u8>,
    pub credential_id: Vec<u8>,
    pub rp_id: String,
}

pub struct DeployedAccountDetails {
    pub address: Address,
    pub unique_account_id: String,
    pub transaction_receipt_json: Option<String>,
}

#[derive(Debug, Clone)]
struct ParsedPasskeyParametersCredential {
    id: String,
    public_key: Vec<u8>,
}

#[derive(Debug, Clone)]
struct ParsedPasskeyParameters {
    credential: ParsedPasskeyParametersCredential,
    expected_origin: String,
}

async fn parse_passkey_parameters(
    params: &PasskeyParameters,
) -> eyre::Result<ParsedPasskeyParameters> {
    let (old_public_key_x, old_public_key_y) =
        extract_public_key(&params.credential_raw_attestation_object)
            .map_err(|e| {
                eyre::eyre!(
                    "XDB deploy_account - Old method - Failed to parse raw attestation object: {}",
                    e
                )
            })?;

    println!(
        "XDB deploy_account - Old method - Passkey public key x: {:?}",
        old_public_key_x
    );
    println!(
        "XDB deploy_account - Old method - Passkey public key y: {:?}",
        old_public_key_y
    );

    let validated = verify_registration(
        &params.credential_raw_attestation_object,
        &params.credential_raw_client_data_json,
        &params.credential_id,
        &params.rp_id,
    )
    .await?;

    let public_key = validated.public_key;
    println!(
        "XDB deploy_account - New method - Passkey public key: {:?}",
        public_key
    );

    let (public_key_x, public_key_y) = {
        let key_bytes = &public_key[public_key.len() - 65..];
        if key_bytes[0] != 0x04 {
            return Err(eyre::eyre!(
                "XDB deploy_account - Invalid public key format from validation"
            ));
        }
        let x_bytes: [u8; 32] = key_bytes[1..33].try_into().unwrap();
        let y_bytes: [u8; 32] = key_bytes[33..65].try_into().unwrap();
        (x_bytes, y_bytes)
    };
    println!(
        "XDB deploy_account - New method - Passkey public key x: {:?}",
        public_key_x
    );
    println!(
        "XDB deploy_account - New method - Passkey public key y: {:?}",
        public_key_y
    );

    println!(
        "XDB deploy_account - Public keys x match: {}",
        old_public_key_x == public_key_x
    );
    println!(
        "XDB deploy_account - Public keys y match: {}",
        old_public_key_y == public_key_y
    );

    let expected_origin = format!("https://{}", params.rp_id);

    use base64::Engine;
    let id_base64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(validated.credential_id);

    Ok(ParsedPasskeyParameters {
        credential: ParsedPasskeyParametersCredential {
            id: id_base64,
            public_key: validated.cose_key_cbor,
        },
        expected_origin,
    })
}

pub async fn deploy_account(
    passkey_parameters: PasskeyParameters,
    config: &Config,
) -> eyre::Result<DeployedAccountDetails> {
    let parsed_params = parse_passkey_parameters(&passkey_parameters).await?;

    println!("XDB deploy_account - parsed_params: {:?}", parsed_params);

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
