use crate::{
    client::contracts::aa_factory::AAFactory,
    config::{contracts::PasskeyContracts, Config},
    utils::{
        alloy::extensions::ProviderExt,
        contract_deployed::{check_contract_deployed, Contract},
        encoding::paymaster::generate_paymaster_input,
    },
};
use alloy::{
    primitives::{Address, Bytes, FixedBytes},
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
use eyre::{eyre, Result};
use rand::RngCore;
use std::fmt::Debug;

pub struct DeployedAccountDetails {
    pub address: Address,
    pub unique_account_id: String,
    pub transaction_receipt: ZKReceiptResponse,
}

#[derive(Debug, Clone)]
pub struct DeployAccountArgs {
    /// Public key of the passkey
    pub credential_public_key: Vec<u8>,

    /// Salt used for the `create2` deployment to make the address deterministic.
    pub salt: Option<[u8; 32]>,

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
            credential_public_key: Vec::new(),
            salt: None,
            expected_origin: None,
            unique_account_id: None,
            paymaster: None,
            contracts: PasskeyContracts::new(
                Address::default(),
                Address::default(),
                Address::default(),
                Address::default(),
            ),
            initial_k1_owners: None,
        }
    }
}

pub async fn deploy_account(
    args: DeployAccountArgs,
    config: &Config,
) -> Result<DeployedAccountDetails> {
    let provider = {
        fn zksync_wallet() -> eyre::Result<ZksyncWallet> {
            let signer = PrivateKeySigner::random();
            let zksync_wallet = ZksyncWallet::from(signer);
            Ok(zksync_wallet)
        }
        let node_url: url::Url = config.clone().node_url;
        let wallet = zksync_wallet().unwrap();
        let provider = zksync_provider()
            .with_recommended_fillers()
            .wallet(wallet.clone())
            .on_http(node_url.clone());
        let wallet_address = wallet.default_signer().address();
        println!("XDB - Wallet address: {}", wallet_address);
        provider
    };

    {
        let account_factory = args.contracts.account_factory;
        println!(
            "XDB deploy_account - Using factory address: {}",
            account_factory
        );

        // Check if factory contract is deployed
        let code = provider.get_code_at(account_factory).await?;
        if code.is_empty() {
            println!("XDB deploy_account - code.len(): {}", code.len());
            return Err(eyre!(
                "Factory contract not deployed at address: {}",
                account_factory
            ));
        }
        println!(
            "XDB deploy_account - Found contract at factory address with bytecode length: {}",
            code.len()
        );
    };

    println!("XDB client::passkey::actions::deploy::deploy_account");
    println!(
        "    XDB Public key (hex): 0x{}",
        hex::encode(&args.credential_public_key)
    );
    println!(
        "    XDB args.credential_public_key: {:?}",
        args.credential_public_key
    );
    println!("    XDB args.salt: {:?}", args.salt);
    println!("    XDB args.expected_origin: {:?}", args.expected_origin);
    println!(
        "XDB deploy_account - args.unique_account_id: {:?}",
        args.unique_account_id
    );
    println!(
        "XDB deploy_account - args.paymaster: {:?}",
        args.paymaster
            .as_ref()
            .map(|p| (p.paymaster, hex::encode(&p.paymaster_input)))
    );
    println!("XDB deploy_account - args.contracts: {:?}", args.contracts);

    let salt: FixedBytes<32> = args
        .salt
        .unwrap_or_else(|| {
            let mut salt = [0u8; 32];
            rand::rng().fill_bytes(&mut salt);
            salt
        })
        .into();
    println!("XDB deploy_account - salt: {:?}", salt);

    let origin = args
        .expected_origin
        .ok_or_else(|| eyre!("Expected origin is required"))?;

    println!("XDB deploy_account - origin: {:?}", origin);

    let (public_key_x, public_key_y) =
        crate::utils::passkey::passkey_signature_from_public_key::get_public_key_bytes_from_passkey_signature(
            &args.credential_public_key,
        )
        .map_err(|e| eyre!("Failed to get public key bytes: {}", e))?;

    println!(
        "XDB deploy_account - passkey public key: ({:?}, {:?})",
        &public_key_x[..4],
        &public_key_y[..4]
    );

    let encoded_passkey_parameters =
        crate::utils::encoding::encode_passkey_module_parameters(
            crate::utils::encoding::PasskeyModuleParams {
                passkey_public_key: (public_key_x, public_key_y),
                expected_origin: origin.clone(),
            },
        )
        .map_err(|e| eyre!("Failed to encode passkey parameters: {}", e))?;

    println!(
        "XDB deploy_account - Encoded passkey parameters length: {}",
        encoded_passkey_parameters.len()
    );

    let encoded_passkey_module_data =
        crate::utils::encoding::encode_module_data(
            crate::utils::encoding::ModuleData {
                address: args.contracts.passkey,
                parameters: encoded_passkey_parameters.clone(),
            },
        )
        .map_err(|e| eyre!("Failed to encode module data: {}", e))?;

    println!(
        "XDB deploy_account - Encoded module data length: {}",
        encoded_passkey_module_data.len()
    );

    let account_id = args
        .unique_account_id
        .unwrap_or_else(|| hex::encode(encoded_passkey_parameters));

    println!("XDB deploy_account - Using account ID: {}", account_id);

    let account_factory = args.contracts.account_factory;
    println!("XDB deploy_account - Using factory address: {}", account_factory);

    check_contract_deployed(
        &config.node_url.clone(),
        &Contract { address: account_factory, name: "AA_FACTORY".to_string() },
    )
    .await?;

    let chain_id = provider.get_chain_id().await?;
    println!("XDB deploy_account - chain_id: {}", chain_id);

    let initial_validators: Vec<Bytes> = vec![encoded_passkey_module_data];
    println!(
        "XDB deploy_account - Initial validators length: {}",
        initial_validators.len()
    );

    let instance = AAFactory::new(account_factory, &provider);

    let initial_k1_owners = args.initial_k1_owners.unwrap_or_default();
    println!("XDB deploy_account - Initial k1 owners: {:?}", initial_k1_owners);

    let deploy_call = instance.deployProxySsoAccount(
        salt,
        account_id.clone(),
        initial_validators.clone(),
        initial_k1_owners.clone(),
    );

    let mut deploy_tx: TransactionRequest =
        deploy_call.into_transaction_request();

    if let Some(mut paymaster) = args.paymaster {
        // If paymaster_input is empty, generate default input
        if paymaster.paymaster_input.is_empty() {
            paymaster.paymaster_input = generate_paymaster_input(None)?;
        }
        deploy_tx = deploy_tx.with_paymaster_params(paymaster);
    }

    println!("XDB deploy_account - Transaction parameters:");
    println!("  Salt: 0x{}", hex::encode(salt));
    println!("  Account ID: {}", account_id);
    println!("  Initial validators: {:?}", initial_validators);
    println!("  Initial k1 owners: {:?}", initial_k1_owners);
    println!(
        "XDB deploy_account - Deploy transaction request: {:?}",
        deploy_tx
    );

    let tx_hash = provider
        .clone()
        .send_transaction(deploy_tx)
        .await
        .map_err(|e| eyre!("Failed to send transaction: {}", e))?
        .tx_hash()
        .to_owned();

    println!("XDB deploy_account - Transaction sent with hash: {}", tx_hash);

    let transaction_receipt =
        provider.wait_for_transaction_receipt(tx_hash).await?;

    println!(
        "XDB deploy_account - Transaction receipt: {:?}",
        transaction_receipt
    );

    let account_created_event =
        get_account_created_event(&transaction_receipt)?;
    let address = account_created_event.accountAddress;
    let unique_account_id = account_created_event.uniqueAccountId;

    println!("XDB deploy_account - Deployed to address: {}", address);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        utils::contract_deployed::{check_contract_deployed, Contract},
        utils::test_utils::{
            spawn_node_and_deploy_contracts, zksync_wallet_from_anvil_zksync,
        },
    };
    use alloy::{
        network::TransactionBuilder,
        primitives::{address, hex, U256},
    };
    use alloy_zksync::{
        network::transaction_request::TransactionRequest,
        provider::zksync_provider,
    };
    use k256::ecdsa::SigningKey;

    #[tokio::test]
    async fn test_deploy_account_with_initial_k1_owners_and_send_transaction(
    ) -> Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;
        let node_url = &config.node_url;

        let mut wallet = zksync_wallet_from_anvil_zksync(&anvil_zksync)?;
        let vitalik = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");

        let wallet_address = wallet.default_signer().address();
        println!("XDB - Wallet address: {}", wallet_address);

        // Deploy the account
        let sample_public_key = vec![
            165, 1, 2, 3, 38, 32, 1, 33, 88, 32, 167, 69, 109, 166, 67, 163,
            110, 143, 71, 60, 77, 232, 220, 7, 121, 156, 141, 24, 71, 28, 210,
            116, 124, 90, 115, 166, 213, 190, 89, 4, 216, 128, 34, 88, 32, 193,
            67, 151, 85, 245, 24, 139, 246, 220, 204, 228, 76, 247, 65, 179,
            235, 81, 41, 196, 37, 216, 117, 201, 244, 128, 8, 73, 37, 195, 20,
            194, 9,
        ];

        // Generate a random account ID
        let unique_account_id = {
            let mut random_bytes = [0u8; 32];
            rand::rng().fill_bytes(&mut random_bytes);
            let id = format!("0x{}", hex::encode(random_bytes));
            println!(
                "XDB - test_deploy_account_with_initial_k1_owners - Generated random account ID: {}",
                id
            );
            id
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

        let args = {
            let paymaster = Some(PaymasterParams {
                paymaster: contracts.account_paymaster,
                paymaster_input: Bytes::new(),
            });
            let origin: String = "https://example.com".to_string();
            DeployAccountArgs {
                credential_public_key: sample_public_key,
                expected_origin: Some(origin),
                unique_account_id: Some(unique_account_id),
                paymaster,
                contracts: contracts.clone(),
                initial_k1_owners: Some(vec![wallet_address]),
                ..Default::default()
            }
        };
        // assert_eq!(provider, 1);
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
            println!("XDB - test_deploy_account_with_initial_k1_owners - Fund receipt: {:?}", receipt);
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
