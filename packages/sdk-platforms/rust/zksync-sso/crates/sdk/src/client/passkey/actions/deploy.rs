use crate::{
    client::contracts::AAFactory,
    config::{Config, contracts::PasskeyContracts},
    utils::{
        alloy::extensions::ProviderExt,
        contract_deployed::{Contract, check_contract_deployed},
        encoding::{
            ModuleData, PasskeyModuleParams, encode_module_data,
            encode_passkey_module_parameters,
            paymaster::generate_paymaster_input,
        },
        passkey::passkey_signature_from_public_key::get_public_key_bytes_from_passkey_signature,
    },
};
use alloy::{
    primitives::{Address, Bytes, FixedBytes, keccak256},
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

pub struct DeployedAccountDetails {
    pub address: Address,
    pub unique_account_id: FixedBytes<32>,
    pub transaction_receipt: ZKReceiptResponse,
}

#[derive(Debug, Clone)]
pub struct CredentialDetails {
    /// Unique id of the passkey public key (base64)
    pub id: String,

    /// Public key of the passkey
    pub public_key: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct DeployAccountArgs {
    /// Credential public key and id
    pub credential: CredentialDetails,

    /// Expected origin of the passkey
    pub expected_origin: Option<String>,

    /// Unique account ID, can be omitted if you don't need it
    pub unique_account_id: Option<String>,

    /// Paymaster used to pay the fees of creating accounts
    pub paymaster: Option<PaymasterParams>,

    /// Contracts
    pub contracts: PasskeyContracts,

    /// Initial K1 owners
    pub initial_k1_owners: Option<Vec<Address>>,
}

impl Default for DeployAccountArgs {
    fn default() -> Self {
        Self {
            credential: CredentialDetails {
                id: String::new(),
                public_key: Vec::new(),
            },
            expected_origin: None,
            unique_account_id: None,
            paymaster: None,
            contracts: PasskeyContracts::default(),
            initial_k1_owners: None,
        }
    }
}

pub async fn deploy_account(
    args: DeployAccountArgs,
    config: &Config,
) -> Result<DeployedAccountDetails> {
    debug!("args.unique_account_id: {:?}", args.unique_account_id.clone());

    let provider = {
        let node_url: url::Url = config.clone().node_url;

        let deploy_wallet = config.clone().deploy_wallet;

        let wallet = ZksyncWallet::from(PrivateKeySigner::from_str(
            &deploy_wallet.private_key_hex,
        )?);

        let provider = zksync_provider()
            .with_recommended_fillers()
            .wallet(wallet.clone())
            .on_http(node_url.clone());
        let wallet_address = wallet.default_signer().address();
        debug!("XDB - Wallet address: {}", wallet_address);

        provider
    };

    {
        let account_factory = args.contracts.account_factory;
        debug!(
            "XDB deploy_account - Using factory address: {}",
            account_factory
        );

        // Check if factory contract is deployed
        let code = provider.get_code_at(account_factory).await?;
        if code.is_empty() {
            debug!("XDB deploy_account - code.len(): {}", code.len());
            return Err(eyre!(
                "Factory contract not deployed at address: {}",
                account_factory
            ));
        }
        debug!(
            "XDB deploy_account - Found contract at factory address with bytecode length: {}",
            code.len()
        );
    };

    debug!("XDB client::passkey::actions::deploy::deploy_account");
    debug!(
        "    XDB Public key (hex): 0x{}",
        hex::encode(&args.credential.public_key)
    );
    debug!(
        "    XDB args.credential.public_key: {:?}",
        args.credential.public_key
    );
    debug!("    XDB args.credential.id: {:?}", args.credential.id);
    debug!("    XDB args.expected_origin: {:?}", args.expected_origin);
    debug!(
        "XDB deploy_account - args.unique_account_id: {:?}",
        args.unique_account_id
    );
    debug!(
        "XDB deploy_account - args.paymaster: {:?}",
        args.paymaster
            .as_ref()
            .map(|p| (p.paymaster, hex::encode(&p.paymaster_input)))
    );
    debug!("XDB deploy_account - args.contracts: {:?}", args.contracts);

    let origin = args
        .expected_origin
        .ok_or_else(|| eyre!("Expected origin is required"))?;

    debug!("XDB deploy_account - origin: {:?}", origin);

    let (public_key_x, public_key_y) =
        get_public_key_bytes_from_passkey_signature(
            &args.credential.public_key,
        )
        .map_err(|e| eyre!("Failed to get public key bytes: {}", e))?;

    debug!(
        "XDB deploy_account - passkey public key: ({:?}, {:?})",
        &public_key_x[..4],
        &public_key_y[..4]
    );

    let encoded_passkey_parameters =
        encode_passkey_module_parameters(PasskeyModuleParams {
            passkey_id: args.credential.id.clone(),
            passkey_public_key: (public_key_x, public_key_y),
            expected_origin: origin.clone(),
        })
        .map_err(|e| eyre!("Failed to encode passkey parameters: {}", e))?;

    debug!(
        "XDB deploy_account - Encoded passkey parameters length: {}",
        encoded_passkey_parameters.len()
    );

    let encoded_passkey_module_data = encode_module_data(ModuleData {
        address: args.contracts.passkey,
        parameters: encoded_passkey_parameters.clone(),
    })
    .map_err(|e| eyre!("Failed to encode module data: {}", e))?;

    debug!(
        "XDB deploy_account - Encoded module data length: {}",
        encoded_passkey_module_data.len()
    );

    let account_id = args
        .unique_account_id
        .map(hex::encode)
        .unwrap_or_else(|| hex::encode(encoded_passkey_parameters));
    debug!("XDB deploy_account - Using account ID: {}", account_id);

    let account_factory = args.contracts.account_factory;
    debug!("XDB deploy_account - Using factory address: {}", account_factory);

    check_contract_deployed(
        &config.node_url.clone(),
        &Contract { address: account_factory, name: "AA_FACTORY".to_string() },
    )
    .await?;

    let chain_id = provider.get_chain_id().await?;
    debug!("XDB deploy_account - chain_id: {}", chain_id);

    let initial_validators: Vec<Bytes> = vec![encoded_passkey_module_data];
    debug!(
        "XDB deploy_account - Initial validators length: {}",
        initial_validators.len()
    );

    let instance = AAFactory::new(account_factory, &provider);

    let initial_k1_owners = args.initial_k1_owners.unwrap_or_default();
    debug!("XDB deploy_account - Initial k1 owners: {:?}", initial_k1_owners);

    let unique_id = hash_unique_account_id(account_id.clone())?;
    debug!("XDB deploy_account - unique_id: {}", unique_id);

    let deploy_call = instance.deployProxySsoAccount(
        unique_id,
        initial_validators.clone(),
        initial_k1_owners.clone(),
    );

    let deploy_tx: TransactionRequest = {
        let mut deploy_tx = deploy_call.into_transaction_request();

        if let Some(mut paymaster) = args.paymaster {
            // If paymaster_input is empty, generate default input
            if paymaster.paymaster_input.is_empty() {
                paymaster.paymaster_input = generate_paymaster_input(None)?;
            }
            deploy_tx = deploy_tx.with_paymaster_params(paymaster);
        }

        deploy_tx
    };

    debug!("XDB deploy_account - Transaction parameters:");
    debug!("  Unique ID Hash: {}", unique_id);
    debug!("  Initial validators: {:?}", initial_validators);
    debug!("  Initial k1 owners: {:?}", initial_k1_owners);
    debug!("XDB deploy_account - Deploy transaction request: {:?}", deploy_tx);

    let tx_hash = provider
        .clone()
        .send_transaction(deploy_tx)
        .await
        .map_err(|e| eyre!("Failed to send transaction: {}", e))?
        .tx_hash()
        .to_owned();

    debug!("XDB deploy_account - Transaction sent with hash: {}", tx_hash);

    let transaction_receipt =
        provider.wait_for_transaction_receipt(tx_hash).await?;

    debug!(
        "XDB deploy_account - Transaction receipt: {:?}",
        transaction_receipt
    );

    let account_created_event =
        get_account_created_event(&transaction_receipt)?;
    let address = account_created_event.accountAddress;
    let unique_account_id = account_created_event.uniqueAccountId;

    debug!("XDB deploy_account - Deployed to address: {}", address);

    Ok(DeployedAccountDetails {
        address,
        unique_account_id,
        transaction_receipt,
    })
}

fn get_account_created_event(
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

fn hash_unique_account_id(
    account_id_hex: String,
) -> eyre::Result<FixedBytes<32>> {
    debug!("XDB hash_unique_account_id - account_id_hex: {:?}", account_id_hex);
    let hash = keccak256(account_id_hex);
    debug!("XDB hash_unique_account_id - hash: {:?}", hash);
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{
        contract_deployed::{Contract, check_contract_deployed},
        test_utils::{
            spawn_node_and_deploy_contracts, zksync_wallet_from_anvil_zksync,
        },
    };
    use alloy::{
        network::TransactionBuilder,
        primitives::{U256, address, hex},
    };
    use alloy_zksync::{
        network::transaction_request::TransactionRequest,
        provider::zksync_provider,
    };
    use k256::ecdsa::SigningKey;
    use rand::RngCore;

    #[tokio::test]
    async fn test_deploy_account() -> Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;
        let node_url = &config.node_url;

        let (wallet, _, _) = zksync_wallet_from_anvil_zksync(&anvil_zksync)?;

        let wallet_address = wallet.default_signer().address();
        println!("XDB - Wallet address: {}", wallet_address);

        let credential_public_key = vec![
            165, 1, 2, 3, 38, 32, 1, 33, 88, 32, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            34, 88, 32, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        ];

        let credential_id = "unique-base64encoded-string".to_string();

        println!(
            "XDB - test_deploy_account_with_initial_k1_owners - Credential ID: {}",
            credential_id
        );

        let deploy_account_credential = CredentialDetails {
            id: credential_id.clone(),
            public_key: credential_public_key,
        };
        let unique_account_id = None;

        let contracts = config.clone().contracts;

        let contract_address = contracts.clone().account_factory;
        {
            let factory_contract = Contract {
                address: contract_address,
                name: "MY_AA_FACTORY".to_string(),
            };
            check_contract_deployed(&node_url.clone(), &factory_contract)
                .await?;
        };

        let origin: String = "https://example.com".to_string();

        let args = {
            let paymaster = Some(PaymasterParams {
                paymaster: contracts.account_paymaster,
                paymaster_input: Bytes::new(),
            });
            DeployAccountArgs {
                credential: deploy_account_credential,
                expected_origin: Some(origin),
                unique_account_id,
                paymaster,
                contracts: contracts.clone(),
                ..Default::default()
            }
        };

        let result = deploy_account(args, &config).await?;

        let deployed_account_address = result.address;

        println!(
            "XDB - test_deploy_account - Deployed account address: {}",
            deployed_account_address
        );

        drop(anvil_zksync);

        Ok(())
    }

    #[tokio::test]
    async fn test_deploy_account_with_initial_k1_owners_and_send_transaction()
    -> Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;
        let node_url = &config.node_url;

        let (mut wallet, _, _) =
            zksync_wallet_from_anvil_zksync(&anvil_zksync)?;
        let vitalik = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");

        let wallet_address = wallet.default_signer().address();
        println!("XDB - Wallet address: {}", wallet_address);

        let credential_public_key = vec![
            165, 1, 2, 3, 38, 32, 1, 33, 88, 32, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            34, 88, 32, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        ];

        let credential_id = "unique-base64encoded-string".to_string();

        println!(
            "XDB - test_deploy_account_with_initial_k1_owners - Credential ID: {}",
            credential_id
        );

        let deploy_account_credential = CredentialDetails {
            id: credential_id,
            public_key: credential_public_key,
        };

        // Generate a random account ID
        let unique_account_id = {
            let mut random_bytes = [0u8; 32];
            rand::rng().fill_bytes(&mut random_bytes);
            let id = hex::encode(random_bytes);
            println!(
                "XDB - test_deploy_account_with_initial_k1_owners - Generated random account ID: {}",
                id
            );
            Some(id)
        };

        let contracts = config.clone().contracts;

        let contract_address = contracts.clone().account_factory;
        {
            let factory_contract = Contract {
                address: contract_address,
                name: "MY_AA_FACTORY".to_string(),
            };
            check_contract_deployed(&node_url.clone(), &factory_contract)
                .await?;
        };

        let origin: String = "https://example.com".to_string();

        let args = {
            let paymaster = Some(PaymasterParams {
                paymaster: contracts.account_paymaster,
                paymaster_input: Bytes::new(),
            });
            DeployAccountArgs {
                credential: deploy_account_credential,
                expected_origin: Some(origin),
                unique_account_id,
                paymaster,
                contracts: contracts.clone(),
                initial_k1_owners: Some(vec![wallet_address]),
            }
        };
        let result = deploy_account(args, &config).await?;
        let deployed_account_address = result.address;
        println!(
            "XDB - test_deploy_account_with_initial_k1_owners - Deployed account address: {}",
            deployed_account_address
        );

        {
            // Register the EOA signer for the deployed account
            let credential =
                SigningKey::from(anvil_zksync.keys()[0].to_owned());
            let address = deployed_account_address;
            use alloy::signers::local::LocalSigner;
            let local_signer =
                LocalSigner::new_with_credential(credential, address, None);
            wallet.register_signer(local_signer);
            let _ = wallet.signer_by_address(address).unwrap();
        };

        let provider = zksync_provider()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(node_url.clone());

        // Check initial balances
        let vitalik_balance_before = provider.get_balance(vitalik).await?;
        println!(
            "XDB - test_deploy_account_with_initial_k1_owners - Vitalik balance before: {}",
            vitalik_balance_before
        );
        // assert_eq!(vitalik_balance_before, U256::ZERO);

        let account_balance_before =
            provider.get_balance(deployed_account_address).await?;
        println!(
            "XDB - test_deploy_account_with_initial_k1_owners - Account balance before: {}",
            account_balance_before
        );

        // Fund the account with 0.1 ETH
        let value = U256::from(100000000000000000u64); // 0.1 ETH
        {
            let fund_tx = TransactionRequest::default()
                .with_to(deployed_account_address)
                .with_value(value);
            let fund_tx_hash =
                provider.send_transaction(fund_tx).await?.tx_hash().to_owned();
            let receipt =
                provider.wait_for_transaction_receipt(fund_tx_hash).await?;
            println!(
                "XDB - test_deploy_account_with_initial_k1_owners - Fund receipt: {:?}",
                receipt
            );
        }
        println!(
            "XDB - test_deploy_account_with_initial_k1_owners - account funded"
        );

        // Verify funding
        let account_balance_after =
            provider.get_balance(deployed_account_address).await?;
        println!("Account balance after funding: {}", account_balance_after);
        assert!(account_balance_after == value);

        // Send ETH from smart account to Vitalik
        let send_amount = U256::from(50000000000000000u64); // 0.05 ETH

        // Create the transaction data
        let tx: TransactionRequest = {
            use alloy::rpc::types::transaction::TransactionRequest as AlloyTransactionRequest;
            use alloy_zksync::network::tx_type::TxType;
            let alloy_tx_request = AlloyTransactionRequest::default()
                .with_from(deployed_account_address)
                .with_to(vitalik)
                .with_value(send_amount);

            let tx_request: TransactionRequest =
                alloy_tx_request.clone().into();
            assert_eq!(tx_request.output_tx_type(), TxType::Eip1559);
            tx_request
        };

        // Send the transaction
        let pending_tx = provider.send_transaction(tx).await?;
        let tx_hash = pending_tx.tx_hash().to_owned();

        // Get receipt
        let receipt = provider.wait_for_transaction_receipt(tx_hash).await?;
        println!("Receipt: {:?}", receipt);

        // Verify final balances
        let vitalik_balance_after = provider.get_balance(vitalik).await?;
        println!("Vitalik balance after: {}", vitalik_balance_after);
        assert_eq!(vitalik_balance_after, send_amount);

        drop(anvil_zksync);

        Ok(())
    }
}
