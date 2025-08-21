use crate::{
    client::passkey::actions::deploy::CredentialDetails,
    config::Config,
    contracts::{AAFactory, SsoAccount},
    utils::{
        alloy::extensions::ProviderExt,
        contract_deployed::{Contract, check_contract_deployed},
        encoding::{
            ModuleData, encode_module_data,
            passkey::{PasskeyModuleParams, encode_passkey_module_parameters},
            paymaster::generate_paymaster_input,
            session::encode_session_key_module_parameters,
        },
        passkey::passkey_signature_from_public_key::get_public_key_bytes_from_passkey_signature,
        session::session_lib::session_spec::SessionSpec,
        viem::strip_0x,
    },
};
use alloy::{
    primitives::{Address, Bytes, FixedBytes, hex, keccak256},
    providers::Provider,
    signers::local::PrivateKeySigner,
    sol_types::SolEvent,
};
use alloy_zksync::{
    network::{
        receipt_response::ReceiptResponse as ZKReceiptResponse,
        transaction_request::TransactionRequest,
        unsigned_tx::eip712::PaymasterParams,
    },
    provider::zksync_provider,
    wallet::ZksyncWallet,
};
use eyre::{Result, eyre};
use log::debug;
use std::{fmt::Debug, str::FromStr};

/// Arguments for configuring the optional session key module to install at deployment time.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SessionModuleArgs {
    /// The module contract address
    pub location: Address,
    /// Optional initial session specification
    pub initial_session: Option<SessionSpec>,
}

/// Arguments for configuring the optional passkey module to install at deployment time.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PasskeyModuleArgs {
    /// Address of the passkey module contract
    pub location: Address,
    /// Passkey credential details (public key + id)
    pub credential: CredentialDetails,
    /// Expected origin used during passkey verification
    pub expected_origin: Option<String>,
}

/// Arguments accepted by [`deploy_modular_account`].
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DeployModularAccountArgs {
    /// Addresses of modules that should be installed without any constructor / parameter data.
    pub install_no_data_modules: Vec<Address>,
    /// List of initial ECDSA owners (can be an empty vector).
    pub owners: Vec<Address>,
    /// Optional session module configuration
    pub session_module: Option<SessionModuleArgs>,
    /// Optional paymaster parameters to cover gas for the deployment
    pub paymaster: Option<PaymasterParams>,
    /// Optional passkey module configuration
    pub passkey_module: Option<PasskeyModuleArgs>,
    /// Optional unique account id. If omitted, it is derived from the unique set of inputs.
    pub unique_account_id: Option<String>,
}

/// Return type of [`deploy_modular_account`].
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DeployModularAccountReturnType {
    /// Deployed smart-account address
    pub address: Address,
    /// Unique account id of the deployed smart-account
    pub unique_account_id: FixedBytes<32>,
    /// Transaction receipt of the deployment
    pub transaction_receipt: ZKReceiptResponse,
}

/// Deploy a modular smart-account on zkSync Era.
///
/// The function mirrors the behaviour of the TypeScript `deployModularAccount` helper
/// found in the JavaScript SDK. It supports the installation of arbitrary modules, an
/// optional paymaster as well as passkey and session modules.
#[allow(dead_code)]
pub async fn deploy_modular_account(
    args: DeployModularAccountArgs,
    config: &Config,
) -> Result<DeployModularAccountReturnType> {
    let node_url: url::Url = config.clone().node_url;
    let wallet = if let Some(deploy_wallet) = config.clone().deploy_wallet {
        ZksyncWallet::from(PrivateKeySigner::from_str(
            &deploy_wallet.private_key_hex,
        )?)
    } else {
        ZksyncWallet::from(PrivateKeySigner::random())
    };
    debug!("XDB deploy_modular_account - wallet: {wallet:?}");
    debug!("XDB deploy_modular_account - node_url: {node_url:?}");
    let wallet_address = wallet.default_signer().address();
    debug!("XDB deploy_modular_account - wallet_address: {wallet_address:?}");

    let account_factory = config.clone().contracts.account_factory;

    //-------------------------------------------------------------------
    // Build provider & wallet
    //-------------------------------------------------------------------
    let provider = {
        zksync_provider()
            .with_recommended_fillers()
            .wallet(wallet.clone())
            .on_http(node_url.clone())
    };

    // --- Step 1: Deploying modular account with initial session (SDK equivalent) ---

    //-------------------------------------------------------------------
    // Basic sanity checks
    //-------------------------------------------------------------------
    // Ensure factory contract is deployed
    check_contract_deployed(
        &config.node_url.clone(),
        &Contract { address: account_factory, name: "AA_FACTORY".into() },
    )
    .await?;

    //-------------------------------------------------------------------
    // Prepare unique id hash
    //-------------------------------------------------------------------
    let mut unique_concat_strings: Vec<String> = Vec::new();

    // 1. Optional explicit unique account id (utf-8 bytes)
    if let Some(uid) = &args.unique_account_id {
        debug!("XDB deploy_modular_account - uid: {uid:?}");
        let uid_bytes = uid.as_bytes();
        debug!("XDB deploy_modular_account - uid_bytes: {uid_bytes:?}");

        let uid_bytes_hex = hex::encode(uid_bytes);
        debug!("XDB deploy_modular_account - uid_bytes_hex: {uid_bytes_hex:?}");
        unique_concat_strings.push(uid_bytes_hex);
    }
    // 2. Owner addresses (20 bytes each)
    debug!("XDB deploy_modular_account - args.owners: {:?}", args.owners);
    for owner in &args.owners {
        debug!("XDB deploy_modular_account - owner: {owner:?}");
        let owner_bytes = owner.as_slice();
        debug!("XDB deploy_modular_account - owner_bytes: {owner_bytes:?}");

        let owner_bytes_hex =
            owner.to_string().split("0x").collect::<Vec<_>>()[1].to_string();
        debug!(
            "XDB deploy_modular_account - owner_bytes_hex: {owner_bytes_hex:?}"
        );
        unique_concat_strings.push(owner_bytes_hex);
    }
    // 3. Passkey credential id if provided (utf-8 bytes)
    if let Some(passkey) = &args.passkey_module {
        debug!("XDB deploy_modular_account - passkey: {passkey:?}");
        let passkey_id_bytes = passkey.credential.id.as_bytes();
        debug!(
            "XDB deploy_modular_account - passkey_id_bytes: {passkey_id_bytes:?}"
        );
        let passkey_id_bytes_hex = hex::encode(passkey_id_bytes);
        debug!(
            "XDB deploy_modular_account - passkey_id_bytes_hex: {passkey_id_bytes_hex:?}"
        );
        unique_concat_strings.push(passkey_id_bytes_hex);
    }

    debug!(
        "XDB deploy_modular_account - unique_concat_strings: {unique_concat_strings:?}"
    );

    let unique_id_concat_strings_joined = unique_concat_strings.join("");
    debug!(
        "XDB deploy_modular_account - unique_id_concat_strings_joined: {unique_id_concat_strings_joined:?}"
    );

    let unique_id_concat_strings_joined_hex =
        alloy::hex::decode(strip_0x(&unique_id_concat_strings_joined))?;
    debug!(
        "XDB deploy_modular_account - unique_id_concat_strings_joined_hex: {unique_id_concat_strings_joined_hex:?}"
    );

    let mut unique_id_concat_hex_bytes: Vec<u8> = vec![];
    if !unique_id_concat_strings_joined.is_empty() {
        // add `0x` prefix
        unique_id_concat_hex_bytes.push(48);
        unique_id_concat_hex_bytes.push(120);
    }

    let unique_id_concat_hex_joined_bytes =
        alloy::hex::decode(&unique_id_concat_strings_joined)?;
    debug!(
        "XDB deploy_modular_account - unique_id_concat_hex_joined_bytes: {unique_id_concat_hex_joined_bytes:?}"
    );
    unique_id_concat_hex_bytes
        .extend_from_slice(&unique_id_concat_hex_joined_bytes);

    debug!(
        "XDB deploy_modular_account - unique_id_concat_hex_bytes: {unique_id_concat_hex_bytes:?}"
    );

    let unique_id_hash: FixedBytes<32> = keccak256(unique_id_concat_hex_bytes);
    debug!("XDB deploy_modular_account - unique_id_hash: {unique_id_hash:?}");

    //-------------------------------------------------------------------
    // Prepare module initialisation data
    //-------------------------------------------------------------------
    let mut modules: Vec<Bytes> = Vec::new();

    // Session module ----------------------------------------------------
    if let Some(session_cfg) = &args.session_module {
        let parameters = if let Some(spec) = session_cfg.initial_session.clone()
        {
            encode_session_key_module_parameters(spec)?
        } else {
            Bytes::new()
        };

        let encoded = encode_module_data(ModuleData {
            address: session_cfg.location,
            parameters,
        })?;
        debug!(
            "XDB deploy_modular_account - encoded session module: {encoded:?}"
        );
        modules.push(encoded);
    }

    // Passkey module ----------------------------------------------------
    if let Some(passkey_cfg) = &args.passkey_module {
        let origin = passkey_cfg.expected_origin.clone().ok_or_else(|| {
            eyre!("expected_origin is required for passkey module")
        })?;

        let (x_pub, y_pub) = get_public_key_bytes_from_passkey_signature(
            &passkey_cfg.credential.public_key,
        )?;

        let passkey_params = PasskeyModuleParams {
            passkey_id: passkey_cfg.credential.id.clone(),
            passkey_public_key: (x_pub, y_pub),
            expected_origin: origin,
        };
        let encoded_params = encode_passkey_module_parameters(passkey_params)?;
        let encoded_module = encode_module_data(ModuleData {
            address: passkey_cfg.location,
            parameters: encoded_params,
        })?;
        modules.push(encoded_module);
    }

    // No-data modules ----------------------------------------------------
    for module_addr in &args.install_no_data_modules {
        let encoded = encode_module_data(ModuleData {
            address: *module_addr,
            parameters: Bytes::new(),
        })?;
        modules.push(encoded);
    }

    debug!("XDB deploy_modular_account - modules: {modules:?}");

    debug!("XDB deploy_modular_account - args.owners: {:?}", args.owners);

    //-------------------------------------------------------------------
    // Build and send the deployment transaction
    //-------------------------------------------------------------------
    let factory_instance = AAFactory::new(account_factory, &provider);

    let deploy_call = factory_instance.deployProxySsoAccount(
        unique_id_hash,
        modules.clone(),
        args.owners.clone(),
    );

    let mut tx_request: TransactionRequest =
        deploy_call.into_transaction_request();

    // Attach paymaster params if provided --------------------------------
    if let Some(mut paymaster) = args.paymaster.clone() {
        if paymaster.paymaster_input.is_empty() {
            paymaster.paymaster_input = generate_paymaster_input(None);
        }
        tx_request = tx_request.with_paymaster_params(paymaster);
    }

    debug!("XDB deploy_modular_account – Tx request: {tx_request:?}");

    let tx_hash = provider
        .clone()
        .send_transaction(tx_request)
        .await
        .map_err(|e| eyre!("Failed to send deployment transaction: {}", e))?
        .tx_hash()
        .to_owned();

    debug!("XDB deploy_modular_account – tx hash: {tx_hash}");

    let receipt = provider.wait_for_transaction_receipt(tx_hash).await?;

    //-------------------------------------------------------------------
    // Parse `AccountCreated` event to obtain account address
    //-------------------------------------------------------------------
    let account_created_event = get_account_created_event(&receipt)?;
    let account_address = account_created_event.accountAddress;
    let unique_account_id = account_created_event.uniqueAccountId;

    debug!("XDB deploy_modular_account – account address: {account_address}");

    Ok(DeployModularAccountReturnType {
        address: account_address,
        unique_account_id,
        transaction_receipt: receipt,
    })
}

pub fn get_account_created_event(
    receipt: &ZKReceiptResponse,
) -> eyre::Result<AAFactory::AccountCreated> {
    let topic = AAFactory::AccountCreated::SIGNATURE_HASH;
    let log = receipt
        .logs()
        .iter()
        .find(|log: &&alloy::rpc::types::Log| log.inner.topics()[0] == topic)
        .ok_or_else(|| eyre!("AccountCreated event not found in logs"))?;
    let event = log.log_decode()?.inner.data;
    Ok(event)
}

pub fn hash_unique_account_id(
    account_id_hex: String,
) -> eyre::Result<FixedBytes<32>> {
    debug!("XDB hash_unique_account_id - account_id_hex: {account_id_hex:?}");
    let hash = keccak256(account_id_hex);
    debug!("XDB hash_unique_account_id - hash: {hash:?}");
    Ok(hash)
}

pub async fn is_module_validator(
    account_address: Address,
    module_address: Address,
    config: &Config,
) -> eyre::Result<bool> {
    let public_provider = {
        let node_url: url::Url = config.clone().node_url;
        zksync_provider().with_recommended_fillers().on_http(node_url)
    };
    let account_contract = SsoAccount::new(account_address, &public_provider);
    let is_module_validator =
        account_contract.isModuleValidator(module_address).call().await?._0;
    Ok(is_module_validator)
}

pub async fn is_k1_owner(
    account_address: Address,
    owner_address: Address,
    config: &Config,
) -> eyre::Result<bool> {
    let public_provider = {
        let node_url: url::Url = config.clone().node_url;
        zksync_provider().with_recommended_fillers().on_http(node_url)
    };
    let account_contract = SsoAccount::new(account_address, &public_provider);
    let is_owner = account_contract.isK1Owner(owner_address).call().await?._0;
    Ok(is_owner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{
        contract_deployed::{Contract, check_contract_deployed},
        test_utils::{
            passkey::get_mock_credential_details,
            spawn_node_and_deploy_contracts, zksync_wallet_from_anvil_zksync,
        },
    };
    use alloy::primitives::address;

    #[tokio::test]
    async fn test_deploy_modular_account() -> Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;
        let node_url = &config.node_url;

        let (wallet, _, _) = zksync_wallet_from_anvil_zksync(&anvil_zksync)?;
        let wallet_address = wallet.default_signer().address();

        let credential = get_mock_credential_details();

        let contracts = config.clone().contracts;
        let account_factory_addr = contracts.account_factory;

        // Ensure factory deployed
        let factory_contract = Contract {
            address: account_factory_addr,
            name: "AA_FACTORY".into(),
        };
        check_contract_deployed(&node_url.clone(), &factory_contract).await?;

        let args = DeployModularAccountArgs {
            install_no_data_modules: vec![],
            owners: vec![wallet_address],
            session_module: None,
            paymaster: None,
            passkey_module: Some(PasskeyModuleArgs {
                location: contracts.passkey,
                credential,
                expected_origin: Some("https://example.com".to_string()),
            }),
            unique_account_id: None,
        };

        // Act
        let result = deploy_modular_account(args, &config).await?;

        // Assert
        println!("Deployed modular account at: {}", result.address);
        assert_ne!(
            result.address,
            address!("0000000000000000000000000000000000000000")
        );

        drop(anvil_zksync);
        Ok(())
    }
}
